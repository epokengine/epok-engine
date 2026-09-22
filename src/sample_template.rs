//! The Sample starter scene and its one behaviour, in all three gameplay
//! flavors. Spinner is deliberately the smallest class the engine can express:
//! it exists so the flavor selector in the Hub is honest for every template,
//! not only for Third Person.
use crate::{
    actor_document::{ClassReference, ComponentInstance},
    blueprint_asset as bp,
    reflection_schema::{ComponentContract, Domain, Type},
    scene::Scene,
    workspace::GameplayFlavor,
};
use std::path::Path;

pub const CLASS_NAME: &str = "Spinner";
pub const CPP_CLASS_ID: &str = "a997b0f2-b3ac-45c2-9d75-9ec11dc890af";
pub const LUA_CLASS_ID: &str = "8731799c-5390-4ab4-9773-cd6b9ac06c15";
pub const BLUEPRINT_CLASS_ID: &str = "86288c8a-613d-4622-b9df-f9439c5e38ce";
pub const LUA_PATH: &str = "assets/scripts/Spinner.lua";
pub const BLUEPRINT_PATH: &str = "assets/Blueprints/Spinner.epokbp";

/// `epok::ActorComponent::tick`. Unannotated reflected declarations are named by
/// their Clang USR, which is stable across machines and checkouts, so a
/// generated project never needs the extractor to bind this override.
const TICK_OVERRIDE: &str =
    "cpp:c:@N@epok@S@ActorComponent@F@tick#$@N@psyqo@S@FixedPoint>#Vi12#I#Vi4096#";

pub fn class_id(flavor: GameplayFlavor) -> &'static str {
    match flavor {
        GameplayFlavor::Cpp => CPP_CLASS_ID,
        GameplayFlavor::Blueprint => BLUEPRINT_CLASS_ID,
        GameplayFlavor::Lua => LUA_CLASS_ID,
    }
}

pub fn create(root: &Path, flavor: GameplayFlavor) -> Result<Scene, String> {
    let mut scene = Scene::default();
    scene.name = "SampleScene".into();
    scene.actors[1].components.push(ComponentInstance::new(
        uuid::Uuid::new_v4(),
        ClassReference::new(CLASS_NAME, class_id(flavor)),
        CLASS_NAME,
    ));
    match flavor {
        GameplayFlavor::Cpp => {
            crate::project::write_changed(
                &root.join("assets/scripts/Spinner.hpp"),
                include_bytes!("../templates/Spinner.hpp"),
            )?;
            crate::project::write_changed(
                &root.join("assets/scripts/Spinner.cpp"),
                b"#include \"Spinner.hpp\"\n",
            )?;
        }
        GameplayFlavor::Lua => {
            crate::project::write_changed(
                &root.join(LUA_PATH),
                include_bytes!("../templates/Spinner.lua"),
            )?;
            let mut registry = crate::lua_identity::Registry::default();
            registry
                .classes
                .insert(LUA_PATH.into(), LUA_CLASS_ID.into());
            registry.save(root)?;
        }
        GameplayFlavor::Blueprint => {
            std::fs::create_dir_all(root.join("assets/Blueprints")).map_err(|e| e.to_string())?;
            bp::create(&root.join(BLUEPRINT_PATH), &blueprint())?;
        }
    }
    Ok(scene)
}

/// Stable ids for the one graph this module authors. A generated asset is data,
/// so it must be byte-identical every time the same template is created.
fn id(what: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(format!("epok.sample.spinner\0{what}").as_bytes());
    let mut bytes: [u8; 16] = digest[..16].try_into().expect("16 bytes");
    bytes[6] = (bytes[6] & 15) | 0x50;
    bytes[8] = (bytes[8] & 63) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}

fn link(node: &str) -> bp::Input {
    bp::Input::Link {
        node: node.into(),
        pin: "value".into(),
    }
}

fn node(id: &str, kind: bp::NodeKind) -> bp::Node {
    bp::Node {
        id: id.into(),
        kind,
        inputs: Default::default(),
        outputs: Default::default(),
    }
}

/// The Blueprint flavor of Spinner: read the owner's rotation, add the turn for
/// this frame to its Y component and write the vector back. Nine nodes, one
/// execution link, and nothing that is not also reachable from C++ and Lua.
pub fn blueprint() -> bp::BlueprintAsset {
    let mut asset = bp::BlueprintAsset::new(
        CLASS_NAME.into(),
        crate::object_model::ACTOR_COMPONENT_ID.into(),
    );
    asset.id = BLUEPRINT_CLASS_ID.into();
    asset.family = Some(crate::reflection_schema::ClassFamily::Component);
    asset.component = Some(ComponentContract {
        owners: [Domain::World3D].into_iter().collect(),
        ..Default::default()
    });
    asset.variables.push(bp::Variable {
        id: id("variable.speed"),
        name: "speed".into(),
        value_type: Type::Fixed,
        default: serde_json::json!(45.0),
        editable: true,
        timeline_animatable: false,
    });

    let (entry, owner, read, x, y, z, speed, turn, sum, make, write) = (
        id("tick.entry"),
        id("tick.owner"),
        id("tick.read"),
        id("tick.x"),
        id("tick.y"),
        id("tick.z"),
        id("tick.speed"),
        id("tick.turn"),
        id("tick.sum"),
        id("tick.make"),
        id("tick.write"),
    );

    let mut entry_node = node(&entry, bp::NodeKind::Entry);
    entry_node
        .outputs
        .insert("next".into(), vec![write.clone()]);

    let owner_node = node(
        &owner,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::GetOwner,
        },
    );
    let mut read_node = node(
        &read,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::GetRotation,
        },
    );
    read_node.inputs.insert("target".into(), link(&owner));

    let component = |index: usize, source: &str| {
        let mut n = node(source, bp::NodeKind::VectorComponent { length: 3, index });
        n.inputs.insert("value".into(), link(&read));
        n
    };
    let x_node = component(0, &x);
    let y_node = component(1, &y);
    let z_node = component(2, &z);

    let speed_node = node(
        &speed,
        bp::NodeKind::GetVariable {
            member: id("variable.speed"),
        },
    );
    let mut turn_node = node(
        &turn,
        bp::NodeKind::Binary {
            op: bp::BinaryOp::Multiply,
        },
    );
    turn_node.inputs.insert("a".into(), link(&speed));
    turn_node.inputs.insert(
        "b".into(),
        bp::Input::Parameter {
            name: "delta_seconds".into(),
        },
    );
    let mut sum_node = node(
        &sum,
        bp::NodeKind::Binary {
            op: bp::BinaryOp::Add,
        },
    );
    sum_node.inputs.insert("a".into(), link(&y));
    sum_node.inputs.insert("b".into(), link(&turn));

    let mut make_node = node(&make, bp::NodeKind::MakeVector { length: 3 });
    make_node.inputs.insert("x".into(), link(&x));
    make_node.inputs.insert("y".into(), link(&sum));
    make_node.inputs.insert("z".into(), link(&z));

    let mut write_node = node(
        &write,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::SetRotation,
        },
    );
    write_node.inputs.insert("target".into(), link(&owner));
    write_node.inputs.insert("value".into(), link(&make));

    let nodes = vec![
        entry_node, owner_node, read_node, x_node, y_node, z_node, speed_node, turn_node, sum_node,
        make_node, write_node,
    ];
    for (index, position) in [
        (&entry, [0., 0.]),
        (&owner, [0., 180.]),
        (&read, [240., 180.]),
        (&x, [480., 120.]),
        (&y, [480., 240.]),
        (&z, [480., 360.]),
        (&speed, [240., 480.]),
        (&turn, [480., 480.]),
        (&sum, [720., 300.]),
        (&make, [960., 240.]),
        (&write, [1200., 0.]),
    ] {
        asset.layout.positions.insert(index.to_string(), position);
    }
    asset
        .layout
        .comments
        .insert(read.clone(), "Current rotation of the owning actor".into());
    asset
        .layout
        .comments
        .insert(turn.clone(), "Degrees to turn this frame".into());

    asset.functions.push(bp::Graph {
        id: id("function.tick"),
        name: "tick".into(),
        override_id: Some(TICK_OVERRIDE.into()),
        timeline: None,
        parameters: vec![crate::reflection_schema::Parameter {
            name: "delta_seconds".into(),
            value_type: Type::Fixed,
            direction: crate::reflection_schema::Direction::Value,
        }],
        returns: Type::Void,
        entry,
        nodes,
    });
    asset
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{CreateOptions, Template};

    /// Sample is the smallest end-to-end proof that the flavor selector is
    /// honest: the same Spinner, generated three ways, compiled three ways.
    #[test]
    fn every_flavor_compiles_the_same_behaviour() {
        for flavor in GameplayFlavor::ALL {
            let parent = std::env::temp_dir().join(format!("epok-sample-{}", uuid::Uuid::new_v4()));
            let root = parent.join("Sample Game");
            crate::workspace::create_with_options(
                &root,
                "Sample Game",
                CreateOptions::new(Template::Sample).with_gameplay(flavor),
            )
            .unwrap_or_else(|e| panic!("{flavor:?} project creation: {e}"));
            let sources = [
                "assets/scripts/Spinner.hpp",
                "assets/scripts/Spinner.cpp",
                LUA_PATH,
                BLUEPRINT_PATH,
            ];
            let mine: &[&str] = match flavor {
                GameplayFlavor::Cpp => &sources[..2],
                GameplayFlavor::Lua => &[LUA_PATH],
                GameplayFlavor::Blueprint => &[BLUEPRINT_PATH],
            };
            for path in sources {
                assert_eq!(
                    root.join(path).is_file(),
                    mine.contains(&path),
                    "{flavor:?} generated {path}"
                );
            }
            let catalog = crate::scripts::catalog(&root)
                .unwrap_or_else(|e| panic!("{flavor:?} script catalog: {e}"));
            let spinner = catalog
                .iter()
                .flat_map(|script| script.classes.iter())
                .find(|class| class.cpp_name == CLASS_NAME)
                .unwrap_or_else(|| panic!("{flavor:?} catalog has no {CLASS_NAME}"));
            assert_eq!(spinner.id, class_id(flavor));
            let speed = spinner
                .properties
                .iter()
                .find(|property| property.name == "speed")
                .unwrap_or_else(|| panic!("{flavor:?} Spinner has no editable speed"));
            assert!(speed.editable);
            let scene = crate::scene::Scene::load(&crate::workspace::startup_scene(&root).unwrap())
                .unwrap();
            assert!(
                crate::project::scene_header(&scene, &catalog)
                    .unwrap()
                    .contains(CLASS_NAME)
            );
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    #[test]
    fn the_blueprint_flavor_is_reproducible() {
        let first = crate::document::to_vec(&blueprint()).unwrap();
        let second = crate::document::to_vec(&blueprint()).unwrap();
        assert_eq!(first, second, "a generated asset must be stable bytes");
        let asset = blueprint();
        let graph = &asset.functions[0];
        for node in &graph.nodes {
            assert!(
                asset.layout.positions.contains_key(&node.id),
                "node {} has no authored position",
                node.id
            );
        }
        assert_eq!(graph.compilation_nodes().count(), graph.nodes.len());
    }
}
