//! The ready-made dungeon key and door, in all three gameplay flavors.
//!
//! A key deactivates its actor when the actor its `player` names enters its trigger
//! (anything, while that names no live actor); a door deactivates once the actor its
//! `key` names is no longer active. Nothing is shared between them but references, so
//! the C++, Lua and Blueprint flavors are the same few lines and need nothing a Lua
//! class cannot express. Both carry the `lock` number the generator assigns, for
//! projects that want it.
use crate::{
    blueprint_asset as bp,
    reflection_schema::{ClassFamily, ComponentContract, Domain, Parameter, Type},
    workspace::GameplayFlavor,
};
use std::path::Path;

pub const KEY: &str = "DungeonKey";
pub const DOOR: &str = "DungeonDoor";
/// The property each class declares for its lock number, the door's key and the
/// key's player.
pub const LOCK: &str = "lock";
pub const KEY_REFERENCE: &str = "key";
pub const PLAYER: &str = "player";

const CPP_KEY_ID: &str = "9dcbb5be-b1da-4a00-a81c-a46a332f5d89";
const CPP_DOOR_ID: &str = "d44f897a-e1f7-4330-9b1e-a5706ec4c032";
const LUA_KEY_ID: &str = "da435669-bde0-4782-b6df-abca91997726";
const LUA_DOOR_ID: &str = "b8285df5-4a19-4b35-a3e3-f9f16d5ef5a5";
const BLUEPRINT_KEY_ID: &str = "f1483d7e-5a4b-4c1e-b072-ff7c3a6d96ea";
const BLUEPRINT_DOOR_ID: &str = "11d781e2-ff89-4a88-823a-e690ec14f761";

const CPP_FILES: [(&str, &[u8]); 4] = [
    (
        "assets/scripts/DungeonKey.hpp",
        include_bytes!("../templates/DungeonKey.hpp"),
    ),
    (
        "assets/scripts/DungeonKey.cpp",
        include_bytes!("../templates/DungeonKey.cpp"),
    ),
    (
        "assets/scripts/DungeonDoor.hpp",
        include_bytes!("../templates/DungeonDoor.hpp"),
    ),
    (
        "assets/scripts/DungeonDoor.cpp",
        include_bytes!("../templates/DungeonDoor.cpp"),
    ),
];
const LUA_KEY_PATH: &str = "assets/scripts/DungeonKey.lua";
const LUA_DOOR_PATH: &str = "assets/scripts/DungeonDoor.lua";
const BLUEPRINT_KEY_PATH: &str = "assets/Blueprints/DungeonKey.epokbp";
const BLUEPRINT_DOOR_PATH: &str = "assets/Blueprints/DungeonDoor.epokbp";

/// `epok::ActorComponent::tick` and `trigger_event`, and `epok::Actor::active`.
const TICK_OVERRIDE: &str =
    "cpp:c:@N@epok@S@ActorComponent@F@tick#$@N@psyqo@S@FixedPoint>#Vi12#I#Vi4096#";
const TRIGGER_EVENT: &str = "45c82d77-d076-43fe-b947-a4eed670f46b";
const ACTOR_ID: &str = "6e6efc67-66c4-4dae-90f8-7c8c4e612dea";
const ACTIVE: &str = "66f0ed92-0e29-498a-aff0-1102dd0c9783";

/// The key and door classes of one flavor, as (name, class id).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Classes {
    pub key: (&'static str, &'static str),
    pub door: (&'static str, &'static str),
}

pub fn classes(flavor: GameplayFlavor) -> Classes {
    let (key, door) = match flavor {
        GameplayFlavor::Cpp => (CPP_KEY_ID, CPP_DOOR_ID),
        GameplayFlavor::Lua => (LUA_KEY_ID, LUA_DOOR_ID),
        GameplayFlavor::Blueprint => (BLUEPRINT_KEY_ID, BLUEPRINT_DOOR_ID),
    };
    Classes {
        key: (KEY, key),
        door: (DOOR, door),
    }
}

/// Whether a class is the key of any flavor.
pub fn is_key(class_id: &str) -> bool {
    [CPP_KEY_ID, LUA_KEY_ID, BLUEPRINT_KEY_ID].contains(&class_id)
}

/// The flavor whose key and door the project already holds, if any: Blueprint, then
/// Lua, then C++.
pub fn installed(root: &Path) -> Option<GameplayFlavor> {
    let has = |paths: &[&str]| paths.iter().all(|p| root.join(p).is_file());
    if has(&[BLUEPRINT_KEY_PATH, BLUEPRINT_DOOR_PATH]) {
        Some(GameplayFlavor::Blueprint)
    } else if has(&[LUA_KEY_PATH, LUA_DOOR_PATH]) {
        Some(GameplayFlavor::Lua)
    } else if has(&[CPP_FILES[0].0, CPP_FILES[2].0]) {
        Some(GameplayFlavor::Cpp)
    } else {
        None
    }
}

/// Key and door files earlier versions wrote, by path and SHA-256. A project file still
/// byte-identical to one of them was never edited, so the current version replaces it.
const SUPERSEDED: [(&str, &str); 5] = [
    // C++ keys and doors sharing a key ring.
    (
        "assets/scripts/DungeonKey.hpp",
        "458082bb9a45785d6b9a0bd65c34e290e9a65ae084a66fcd77976c56535fc1c3",
    ),
    (
        "assets/scripts/DungeonDoor.hpp",
        "1eca3cfbdf022c89be5932a138311d263eee652841480aefa5c2ca19e76101e8",
    ),
    // Keys anything picked up.
    (
        "assets/scripts/DungeonKey.hpp",
        "93a3fcba0dd407de636f71f19ad793722fb544a38a3b39ec49810b52646ebb94",
    ),
    (
        LUA_KEY_PATH,
        "f813d10bb772cc823b2b0a1d6782c77db3f92cf44c85e1865ffda808a5cfc419",
    ),
    (
        BLUEPRINT_KEY_PATH,
        "ad0ef7080ce7b43aef9f966c9eee7f3301da016efa0ecfc401bf8ed14f54157c",
    ),
];

fn superseded(path: &str, bytes: &[u8]) -> bool {
    use sha2::{Digest, Sha256};
    let digest = format!("{:x}", Sha256::digest(bytes));
    SUPERSEDED
        .iter()
        .any(|(old, hash)| *old == path && *hash == digest)
}

/// What `install` wrote: files the project lacked, and unedited earlier versions it
/// brought up to date.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Written {
    pub added: Vec<&'static str>,
    pub updated: Vec<&'static str>,
}

/// The key and door the project uses: the flavor it holds, or C++ written now. An
/// unedited earlier version of the held flavor is brought up to date.
pub fn ensure(root: &Path) -> Result<(Classes, Written), String> {
    let flavor = installed(root).unwrap_or(GameplayFlavor::Cpp);
    Ok((classes(flavor), install(root, flavor)?))
}

/// Writes the key and door of `flavor` into the project. Edited files are kept, so a
/// project's own changes survive; missing ones are added and unedited earlier
/// versions replaced.
pub fn install(root: &Path, flavor: GameplayFlavor) -> Result<Written, String> {
    let mut written = Written::default();
    let mut write = |path: &'static str, bytes: &[u8]| -> Result<(), String> {
        let target = crate::assets::inside(root, path)?;
        if target.exists() {
            let current = std::fs::read(&target).map_err(|e| e.to_string())?;
            if !superseded(path, &current) {
                return Ok(());
            }
            written.updated.push(path);
        } else {
            written.added.push(path);
        }
        crate::project::write_changed(&target, bytes)
    };
    match flavor {
        GameplayFlavor::Cpp => {
            for (path, bytes) in CPP_FILES {
                write(path, bytes)?;
            }
        }
        GameplayFlavor::Lua => {
            write(LUA_KEY_PATH, include_bytes!("../templates/DungeonKey.lua"))?;
            write(
                LUA_DOOR_PATH,
                include_bytes!("../templates/DungeonDoor.lua"),
            )?;
            // Other templates' Lua classes keep their identities.
            let mut registry = crate::lua_identity::read(root).unwrap_or_default();
            registry
                .classes
                .insert(LUA_KEY_PATH.into(), LUA_KEY_ID.into());
            registry
                .classes
                .insert(LUA_DOOR_PATH.into(), LUA_DOOR_ID.into());
            registry.save(root)?;
        }
        GameplayFlavor::Blueprint => {
            for (path, asset) in [
                (BLUEPRINT_KEY_PATH, key_blueprint()),
                (BLUEPRINT_DOOR_PATH, door_blueprint()),
            ] {
                write(
                    path,
                    &crate::document::to_vec(&asset).map_err(|e| e.to_string())?,
                )?;
            }
        }
    }
    Ok(written)
}

// ---- Blueprint flavor ------------------------------------------------------------------------

/// Stable ids, so a generated asset is the same bytes every time.
fn id(class: &str, what: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(format!("epok.dungeon.{class}\0{what}").as_bytes());
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

fn component(name: &str, class_id: &str) -> bp::BlueprintAsset {
    let mut asset =
        bp::BlueprintAsset::new(name.into(), crate::object_model::ACTOR_COMPONENT_ID.into());
    asset.id = class_id.into();
    asset.family = Some(ClassFamily::Component);
    asset.component = Some(ComponentContract {
        owners: [Domain::World3D, Domain::World2D].into_iter().collect(),
        ..Default::default()
    });
    asset.variables.push(bp::Variable {
        id: id(name, "variable.lock"),
        name: LOCK.into(),
        value_type: Type::UInt32,
        default: serde_json::json!(0),
        editable: true,
        timeline_animatable: false,
    });
    asset
}

/// A node that deactivates the owning actor, run from `from`'s `pin`.
fn deactivate_owner(
    class: &str,
    nodes: &mut Vec<bp::Node>,
    from: &mut bp::Node,
    pin: &str,
) -> (String, String) {
    let (owner, hide) = (id(class, "owner"), id(class, "hide"));
    nodes.push(node(
        &owner,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::GetOwner,
        },
    ));
    let mut hide_node = node(
        &hide,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::SetActive,
        },
    );
    hide_node.inputs.insert("target".into(), link(&owner));
    hide_node.inputs.insert(
        "active".into(),
        bp::Input::Literal {
            value_type: Type::Bool,
            value: serde_json::json!(false),
        },
    );
    from.outputs.insert(pin.into(), vec![hide.clone()]);
    nodes.push(hide_node);
    (owner, hide)
}

/// An editable actor reference, empty by default.
fn actor_variable(class: &str, name: &str) -> (String, bp::Variable) {
    let member = id(class, &format!("variable.{name}"));
    let variable = bp::Variable {
        id: member.clone(),
        name: name.into(),
        value_type: Type::ActorRef {
            class: Some(ACTOR_ID.into()),
        },
        default: serde_json::Value::Null,
        editable: true,
        timeline_animatable: false,
    };
    (member, variable)
}

fn binary(id: &str, op: bp::BinaryOp, a: bp::Input, b: bp::Input) -> bp::Node {
    let mut binary = node(id, bp::NodeKind::Binary { op });
    binary.inputs.insert("a".into(), a);
    binary.inputs.insert("b".into(), b);
    binary
}

/// The Blueprint key: when its player (or, while none is alive, anything) enters the
/// trigger, deactivate the owner.
pub fn key_blueprint() -> bp::BlueprintAsset {
    let mut asset = component(KEY, BLUEPRINT_KEY_ID);
    let (player_variable, variable) = actor_variable(KEY, PLAYER);
    asset.variables.push(variable);
    let phase = Type::Enum {
        cpp_name: "epok::TriggerPhase".into(),
        variants: [("Enter", 0), ("Stay", 1), ("Exit", 2)]
            .into_iter()
            .map(|(n, v)| (n.to_string(), v))
            .collect(),
    };
    let [
        entry,
        entered,
        player,
        named,
        anyone,
        same,
        allowed,
        picked,
        branch,
    ] = [
        "entry", "entered", "player", "named", "anyone", "same", "allowed", "picked", "branch",
    ]
    .map(|what| id(KEY, what));
    let parameter = |name: &str| bp::Input::Parameter { name: name.into() };
    let mut nodes = Vec::new();
    let mut entry_node = node(&entry, bp::NodeKind::Entry);
    entry_node
        .outputs
        .insert("next".into(), vec![branch.clone()]);
    let entered_node = binary(
        &entered,
        bp::BinaryOp::Equal,
        parameter("phase"),
        bp::Input::Literal {
            value_type: phase.clone(),
            value: serde_json::json!(0),
        },
    );
    let player_node = node(
        &player,
        bp::NodeKind::GetVariable {
            member: player_variable,
        },
    );
    let mut named_node = node(
        &named,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::IsValid,
        },
    );
    named_node.inputs.insert("target".into(), link(&player));
    let mut anyone_node = node(&anyone, bp::NodeKind::Not);
    anyone_node.inputs.insert("value".into(), link(&named));
    let same_node = binary(
        &same,
        bp::BinaryOp::Equal,
        parameter("other"),
        link(&player),
    );
    let allowed_node = binary(&allowed, bp::BinaryOp::Or, link(&anyone), link(&same));
    let picked_node = binary(&picked, bp::BinaryOp::And, link(&entered), link(&allowed));
    let mut branch_node = node(&branch, bp::NodeKind::Branch);
    branch_node.inputs.insert("condition".into(), link(&picked));
    let (owner, hide) = deactivate_owner(KEY, &mut nodes, &mut branch_node, "true");
    nodes.extend([
        entry_node,
        entered_node,
        player_node,
        named_node,
        anyone_node,
        same_node,
        allowed_node,
        picked_node,
        branch_node,
    ]);
    for (node, position) in [
        (&entry, [0., 0.]),
        (&entered, [260., 140.]),
        (&player, [0., 340.]),
        (&named, [260., 280.]),
        (&same, [260., 420.]),
        (&anyone, [520., 280.]),
        (&allowed, [780., 340.]),
        (&picked, [1040., 180.]),
        (&branch, [1300., 0.]),
        (&owner, [1300., 200.]),
        (&hide, [1560., 0.]),
    ] {
        asset.layout.positions.insert(node.clone(), position);
    }
    asset.layout.comments.insert(
        allowed.clone(),
        "Only the player, or anything while no player is alive".into(),
    );
    asset
        .layout
        .comments
        .insert(branch.clone(), "Picked up the moment it is entered".into());
    asset.functions.push(bp::Graph {
        id: id(KEY, "function.trigger_event"),
        name: "trigger_event".into(),
        override_id: Some(TRIGGER_EVENT.into()),
        timeline: None,
        parameters: vec![
            Parameter {
                name: "other".into(),
                value_type: Type::ObjectRef { class: None },
                direction: crate::reflection_schema::Direction::Value,
            },
            Parameter {
                name: "phase".into(),
                value_type: phase,
                direction: crate::reflection_schema::Direction::Value,
            },
        ],
        returns: Type::Void,
        entry,
        nodes,
    });
    asset
}

/// The Blueprint door: every frame, if its key is set and no longer active,
/// deactivate the owner.
pub fn door_blueprint() -> bp::BlueprintAsset {
    let mut asset = component(DOOR, BLUEPRINT_DOOR_ID);
    let (key_variable, variable) = actor_variable(DOOR, KEY_REFERENCE);
    asset.variables.push(variable);
    let [entry, key, valid, has_key, active, taken, gone] = [
        "entry", "key", "valid", "has_key", "active", "taken", "gone",
    ]
    .map(|what| id(DOOR, what));
    let mut nodes = Vec::new();
    let mut entry_node = node(&entry, bp::NodeKind::Entry);
    entry_node
        .outputs
        .insert("next".into(), vec![has_key.clone()]);
    let key_node = node(
        &key,
        bp::NodeKind::GetVariable {
            member: key_variable,
        },
    );
    let mut valid_node = node(
        &valid,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::IsValid,
        },
    );
    valid_node.inputs.insert("target".into(), link(&key));
    let mut has_key_node = node(&has_key, bp::NodeKind::Branch);
    has_key_node.inputs.insert("condition".into(), link(&valid));
    has_key_node
        .outputs
        .insert("true".into(), vec![gone.clone()]);
    let mut active_node = node(
        &active,
        bp::NodeKind::CallOn {
            class: ACTOR_ID.into(),
            function: ACTIVE.into(),
        },
    );
    active_node.inputs.insert("__target".into(), link(&key));
    let mut taken_node = node(&taken, bp::NodeKind::Not);
    taken_node.inputs.insert("value".into(), link(&active));
    let mut gone_node = node(&gone, bp::NodeKind::Branch);
    gone_node.inputs.insert("condition".into(), link(&taken));
    let (owner, hide) = deactivate_owner(DOOR, &mut nodes, &mut gone_node, "true");
    nodes.extend([
        entry_node,
        key_node,
        valid_node,
        has_key_node,
        active_node,
        taken_node,
        gone_node,
    ]);
    for (node, position) in [
        (&entry, [0., 0.]),
        (&key, [0., 180.]),
        (&valid, [240., 180.]),
        (&has_key, [240., 0.]),
        (&active, [480., 180.]),
        (&taken, [720., 180.]),
        (&gone, [720., 0.]),
        (&owner, [720., 320.]),
        (&hide, [960., 0.]),
    ] {
        asset.layout.positions.insert(node.clone(), position);
    }
    asset
        .layout
        .comments
        .insert(gone.clone(), "Open once the key has been picked up".into());
    asset.functions.push(bp::Graph {
        id: id(DOOR, "function.tick"),
        name: "tick".into(),
        override_id: Some(TICK_OVERRIDE.into()),
        timeline: None,
        parameters: vec![Parameter {
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

    /// Each flavor's key and door go through the real compilers and publish the
    /// properties the generator writes.
    #[test]
    fn every_flavor_compiles_the_key_and_the_door() {
        for flavor in GameplayFlavor::ALL {
            let parent =
                std::env::temp_dir().join(format!("epok-dungeon-keys-{}", uuid::Uuid::new_v4()));
            let root = parent.join("Keys");
            crate::workspace::create_with_options(
                &root,
                "Keys",
                CreateOptions::new(Template::Basic),
            )
            .unwrap();
            assert_eq!(installed(&root), None);
            install(&root, flavor).unwrap();
            assert_eq!(installed(&root), Some(flavor), "{flavor:?}");
            let catalog = crate::scripts::catalog(&root)
                .unwrap_or_else(|e| panic!("{flavor:?} script catalog: {e}"));
            let expected = classes(flavor);
            for ((name, class_id), properties) in [
                (expected.key, &[LOCK, PLAYER][..]),
                (expected.door, &[LOCK, KEY_REFERENCE][..]),
            ] {
                let class = catalog
                    .iter()
                    .flat_map(|script| script.classes.iter())
                    .find(|class| class.cpp_name == name)
                    .unwrap_or_else(|| panic!("{flavor:?} catalog has no {name}"));
                assert_eq!(class.id, class_id, "{flavor:?} {name}");
                for property in properties {
                    assert!(
                        class
                            .properties
                            .iter()
                            .any(|p| p.name == *property && p.editable),
                        "{flavor:?} {name} has no editable {property}"
                    );
                }
            }
            // The bodies compile, not only the declarations.
            let native = crate::scripts::native_catalog(&root).unwrap();
            match flavor {
                GameplayFlavor::Lua => {
                    crate::scripts::compile_lua(&root, &native)
                        .unwrap_or_else(|e| panic!("Lua: {e}"))
                        .expect("two Lua classes");
                }
                GameplayFlavor::Blueprint => {
                    crate::scripts::compile_blueprints(&root, &native)
                        .unwrap_or_else(|e| panic!("Blueprint: {e}"))
                        .expect("two Blueprint classes");
                }
                GameplayFlavor::Cpp => {}
            }
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    #[test]
    fn generated_blueprints_are_stable_and_laid_out() {
        for make in [key_blueprint, door_blueprint] {
            assert_eq!(
                crate::document::to_vec(&make()).unwrap(),
                crate::document::to_vec(&make()).unwrap()
            );
            let asset = make();
            for graph in &asset.functions {
                for node in &graph.nodes {
                    assert!(asset.layout.positions.contains_key(&node.id));
                }
            }
        }
    }
}
