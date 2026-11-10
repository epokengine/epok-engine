//! The ready-made dungeon key and door, in all three gameplay flavors.
//!
//! Fresh classes extend the shared pickup/door mechanisms and bind explicit
//! ownership flags. Project callbacks implement automatic opening and visuals.
//! Existing classes retain their legacy actor-reference contract and source bytes.
use crate::{
    actor_document::ClassReference,
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
pub const LOCK_ID: &str = "lock_id";
pub const PICKUP_BASE: &str = "117a8e4b-4df8-53da-b42e-58be2f3cae4f";
pub const DOOR_BASE: &str = "fd68c79f-403a-55f2-8f8f-2ef861166923";
pub const FLAG_BASE: &str = "397c4a8e-a044-59b5-ab51-a54d214f1789";
pub fn default_key_state() -> ClassReference {
    ClassReference::new("epok::GameplayFlagComponent", FLAG_BASE)
}
pub fn key_state_id(actor: uuid::Uuid) -> uuid::Uuid {
    uuid::Uuid::parse_str(&id(&actor.to_string(), "collected-state")).expect("derived UUID")
}
pub fn rule_component_id(actor: uuid::Uuid, key: bool) -> uuid::Uuid {
    uuid::Uuid::parse_str(&id(
        &actor.to_string(),
        if key { "key-rule" } else { "door-rule" },
    ))
    .expect("derived UUID")
}

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

/// The inherited tick slot remains stable across native and project subclasses.
const TICK_OVERRIDE: &str =
    "cpp:c:@N@epok@S@ActorComponent@F@tick#$@N@psyqo@S@FixedPoint>#Vi12#I#Vi4096#";

/// The key and door classes of one flavor, as (name, class id).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Classes {
    pub key: (&'static str, &'static str),
    pub door: (&'static str, &'static str),
}

/// Resolved project roles. UUIDs remain authoritative when a class is renamed
/// or replaced by a compatible project subclass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    pub key: ClassReference,
    pub door: ClassReference,
    pub key_state: ClassReference,
    pub shared_key: bool,
    pub shared_door: bool,
}
impl From<Classes> for Bindings {
    fn from(classes: Classes) -> Self {
        Self {
            key: ClassReference::new(classes.key.0, classes.key.1),
            door: ClassReference::new(classes.door.0, classes.door.1),
            key_state: default_key_state(),
            shared_key: true,
            shared_door: true,
        }
    }
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

/// Compatibility detection for projects without template installation metadata.
/// Recorded project roles and gameplay choices take precedence in `ensure`.
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

/// Files installed by this operation. Existing sources are never updated
/// implicitly across a behavior/property-contract change.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Written {
    pub added: Vec<&'static str>,
    pub updated: Vec<&'static str>,
}

/// Resolve semantic roles, adopting older installation records on first use.
/// File detection remains only for projects predating installation metadata.
#[cfg(test)]
pub fn ensure(root: &Path) -> Result<(Bindings, Written), String> {
    ensure_in(root, None)
}

pub fn ensure_for_domain(root: &Path, domain: Domain) -> Result<(Bindings, Written), String> {
    ensure_in(root, Some(domain))
}

fn ensure_in(root: &Path, domain: Option<Domain>) -> Result<(Bindings, Written), String> {
    let mut transaction = crate::helper_install::Transaction::new(root);
    let Some(installation) = crate::template_install::read(root)? else {
        let flavor = installed(root).unwrap_or(GameplayFlavor::Cpp);
        let written = install_in(root, flavor, &mut transaction)?;
        let bindings = resolve(root, classes(flavor).into(), domain)?;
        transaction.commit();
        return Ok((bindings, written));
    };
    match (installation.roles.get("key"), installation.roles.get("door")) {
        (Some(key), Some(door)) => {
            let mut bindings: Bindings = classes(GameplayFlavor::Cpp).into();
            bindings.key = key.clone();
            bindings.door = door.clone();
            bindings.key_state = installation.roles.get("key_state").cloned().unwrap_or_else(default_key_state);
            Ok((resolve(root, bindings, domain)?, Written::default()))
        }
        (None, None) => {
            let flavor = GameplayFlavor::parse(&installation.gameplay).ok_or("Invalid recorded dungeon gameplay flavor")?;
            let written = install_in(root, flavor, &mut transaction)?;
            let bindings = resolve(root, classes(flavor).into(), domain)?;
            let template = crate::workspace::Template::parse(&installation.template).ok_or("Invalid recorded template")?;
            let mut roles = installation.roles;
            roles.insert("key".into(), bindings.key.clone());
            roles.insert("door".into(), bindings.door.clone());
            if bindings.shared_key {
                roles.insert("key_state".into(), bindings.key_state.clone());
            }
            crate::template_install::write(root, template, flavor,
                roles.iter().map(|(role, class)| (role.as_str(), class.clone())))?;
            transaction.commit();
            Ok((bindings, written))
        }
        _ => Err("Dungeon regeneration requires both key and door class roles. The installation is preserved.".into()),
    }
}

fn resolve(root: &Path, bindings: Bindings, domain: Option<Domain>) -> Result<Bindings, String> {
    // Project creation can resolve roles before the manifest/execution context
    // exists. Use the authoritative declaration pass; Build/Play compiles bodies
    // after that context has been written.
    let registry = crate::scripts::declaration_registry(root)?;
    let model = registry.model().map_err(|errors| {
        errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let role = |name: &str,
                reference: &ClassReference,
                base: &str,
                legacy: &str,
                shared: &str|
     -> Result<(ClassReference, bool), String> {
        let class = reference
            .resolve(&model)
            .ok_or_else(|| format!("Dungeon {name} role refers to a missing class UUID."))?;
        if class.family != ClassFamily::Component
            || class.abstract_class
            || class.component.as_ref().is_none_or(|contract| {
                !contract
                    .owners
                    .iter()
                    .any(|domain| matches!(domain, Domain::World3D | Domain::World2D))
            })
        {
            return Err(format!(
                "Dungeon {name} role must be a concrete world component."
            ));
        }
        if domain.is_some_and(|domain| !class.component.as_ref().unwrap().owners.contains(&domain))
        {
            return Err(format!(
                "Dungeon {name} role does not support the generated world domain."
            ));
        }
        let properties = registry.properties(&class.cpp_name);
        let inherited = class.ancestry.iter().any(|id| id == base);
        let lock_property = if inherited { LOCK_ID } else { LOCK };
        let reference_property = if inherited { shared } else { legacy };
        if !properties
            .iter()
            .any(|p| p.name == lock_property && p.value_type == Type::UInt32)
            || !properties.iter().any(|p| {
                p.name == reference_property
                    && matches!(
                        p.value_type,
                        Type::ActorRef { .. } | Type::ObjectRef { .. } | Type::ComponentRef { .. }
                    )
            })
        {
            return Err(format!(
                "Dungeon {name} role requires {lock_property} and {reference_property} properties."
            ));
        }
        Ok((ClassReference::new(&class.cpp_name, &class.id), inherited))
    };
    let (key, shared_key) = role(
        "key",
        &bindings.key,
        PICKUP_BASE,
        PLAYER,
        "accepted_initiator",
    )?;
    let (door, shared_door) = role(
        "door",
        &bindings.door,
        DOOR_BASE,
        KEY_REFERENCE,
        "requirement_flag",
    )?;
    if shared_door && !shared_key {
        return Err(
            "A shared Dungeon door requires an explicit ownership flag from a shared pickup key."
                .into(),
        );
    }
    let mut key_state = bindings.key_state;
    if shared_key {
        let flag = key_state
            .resolve(&model)
            .ok_or("Dungeon key_state role refers to a missing class UUID")?;
        if flag.abstract_class || !flag.ancestry.iter().any(|id| id == FLAG_BASE) {
            return Err("Dungeon key_state role must derive from GameplayFlagComponent".into());
        }
        if domain.is_some_and(|domain| {
            flag.component
                .as_ref()
                .is_none_or(|c| !c.owners.contains(&domain))
        }) {
            return Err(
                "Dungeon key_state role does not support the generated world domain.".into(),
            );
        }
        key_state = ClassReference::new(&flag.cpp_name, &flag.id);
    }
    Ok(Bindings {
        key,
        door,
        key_state,
        shared_key,
        shared_door,
    })
}

/// Install missing sources only. Existing implementations, including old
/// untouched versions, retain their associated saved-property contract.
pub fn install(root: &Path, flavor: GameplayFlavor) -> Result<Written, String> {
    let mut transaction = crate::helper_install::Transaction::new(root);
    let written = install_in(root, flavor, &mut transaction)?;
    transaction.commit();
    Ok(written)
}

fn install_in(
    root: &Path,
    flavor: GameplayFlavor,
    transaction: &mut crate::helper_install::Transaction,
) -> Result<Written, String> {
    // Validate identity data before creating any sources. Failed resolution in
    // ensure also restores registry changes and removes only our own files.
    let registry = (flavor == GameplayFlavor::Lua)
        .then(|| crate::lua_identity::read(root))
        .transpose()?;
    let mut written = Written::default();
    let mut write = |path: &'static str, bytes: &[u8]| -> Result<(), String> {
        if transaction.create(path, bytes)? {
            written.added.push(path);
        }
        Ok(())
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
            let mut registry = registry.expect("validated Lua registry");
            registry
                .classes
                .entry(LUA_KEY_PATH.into())
                .or_insert_with(|| LUA_KEY_ID.into());
            registry
                .classes
                .entry(LUA_DOOR_PATH.into())
                .or_insert_with(|| LUA_DOOR_ID.into());
            registry.validate()?;
            transaction.settings(
                crate::lua_identity::REGISTRY,
                &crate::document::to_vec(&registry)?,
            )?;
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
    let mut asset = bp::BlueprintAsset::new(
        name.into(),
        if name == KEY { PICKUP_BASE } else { DOOR_BASE }.into(),
    );
    asset.id = class_id.into();
    asset.family = Some(ClassFamily::Component);
    asset.component = Some(ComponentContract {
        owners: [Domain::World3D, Domain::World2D].into_iter().collect(),
        ..Default::default()
    });
    asset.variables.push(bp::Variable {
        id: id(name, "variable.lock_id"),
        name: LOCK_ID.into(),
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

fn binary(id: &str, op: bp::BinaryOp, a: bp::Input, b: bp::Input) -> bp::Node {
    let mut binary = node(id, bp::NodeKind::Binary { op });
    binary.inputs.insert("a".into(), a);
    binary.inputs.insert("b".into(), b);
    binary
}

/// Editable project outcome over native pickup admission and ownership.
pub fn key_blueprint() -> bp::BlueprintAsset {
    let mut asset = component(KEY, BLUEPRINT_KEY_ID);
    let entry = id(KEY, "collected.entry");
    let mut first = node(&entry, bp::NodeKind::Entry);
    let mut nodes = Vec::new();
    deactivate_owner(KEY, &mut nodes, &mut first, "next");
    nodes.push(first);
    for (i, n) in nodes.iter().enumerate() {
        asset
            .layout
            .positions
            .insert(n.id.clone(), [i as f32 * 240., 0.]);
    }
    asset.functions.push(bp::Graph {
        id: id(KEY, "function.collected"),
        name: "collected".into(),
        override_id: Some(
            "cpp:c:@N@epok@S@PickupComponent@F@collected#$@N@epok@S@ObjectId#i#".into(),
        ),
        timeline: None,
        parameters: vec![
            Parameter {
                name: "initiator".into(),
                value_type: Type::ObjectRef { class: None },
                direction: crate::reflection_schema::Direction::Value,
            },
            Parameter {
                name: "port".into(),
                value_type: Type::UInt32,
                direction: crate::reflection_schema::Direction::Value,
            },
        ],
        returns: Type::Void,
        entry,
        nodes,
    });
    asset
}

/// Automatic opening is project policy; native DoorComponent applies collision.
pub fn door_blueprint() -> bp::BlueprintAsset {
    let mut asset = component(DOOR, BLUEPRINT_DOOR_ID);
    let [
        entry,
        flag,
        valid,
        ready,
        allowed,
        branch,
        owner,
        open,
        parent,
    ] = [
        "tick.entry",
        "tick.flag",
        "tick.valid",
        "tick.ready",
        "tick.allowed",
        "tick.branch",
        "tick.owner",
        "tick.open",
        "tick.parent",
    ]
    .map(|s| id(DOOR, s));
    let mut first = node(&entry, bp::NodeKind::Entry);
    first.outputs.insert("next".into(), vec![branch.clone()]);
    let flag_node = node(
        &flag,
        bp::NodeKind::GetVariable {
            member: "cpp:c:@N@epok@S@DoorComponent@FI@requirement_flag".into(),
        },
    );
    let mut valid_node = node(
        &valid,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::IsValid,
        },
    );
    valid_node.inputs.insert("target".into(), link(&flag));
    let ready_node = node(
        &ready,
        bp::NodeKind::Call {
            function: "cpp:c:@N@epok@S@DoorComponent@F@requirement_met#1".into(),
        },
    );
    let allowed_node = binary(&allowed, bp::BinaryOp::And, link(&valid), link(&ready));
    let mut branch_node = node(&branch, bp::NodeKind::Branch);
    branch_node
        .inputs
        .insert("condition".into(), link(&allowed));
    branch_node
        .outputs
        .insert("true".into(), vec![open.clone()]);
    branch_node
        .outputs
        .insert("false".into(), vec![parent.clone()]);
    let owner_node = node(
        &owner,
        bp::NodeKind::Builtin {
            operation: bp::Builtin::GetOwner,
        },
    );
    let mut open_node = node(
        &open,
        bp::NodeKind::Call {
            function: "cpp:c:@N@epok@S@DoorComponent@F@open#$@N@epok@S@ObjectId#".into(),
        },
    );
    open_node.inputs.insert("initiator".into(), link(&owner));
    open_node
        .outputs
        .insert("next".into(), vec![parent.clone()]);
    let mut parent_node = node(&parent, bp::NodeKind::CallParent);
    parent_node.inputs.insert(
        "delta_seconds".into(),
        bp::Input::Parameter {
            name: "delta_seconds".into(),
        },
    );
    let nodes = vec![
        first,
        flag_node,
        valid_node,
        ready_node,
        allowed_node,
        branch_node,
        owner_node,
        open_node,
        parent_node,
    ];
    for (i, n) in nodes.iter().enumerate() {
        asset
            .layout
            .positions
            .insert(n.id.clone(), [(i % 3) as f32 * 260., (i / 3) as f32 * 180.]);
    }
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
    let phase = Type::Enum {
        cpp_name: "epok::DoorState".into(),
        variants: [
            ("Closed".into(), 0),
            ("Open".into(), 1),
            ("Locked".into(), 2),
        ]
        .into(),
    };
    let entry = id(DOOR, "changed.entry");
    let branch = id(DOOR, "changed.branch");
    let opened = id(DOOR, "changed.opened");
    let mut first = node(&entry, bp::NodeKind::Entry);
    first.outputs.insert("next".into(), vec![branch.clone()]);
    let opened_node = binary(
        &opened,
        bp::BinaryOp::Equal,
        bp::Input::Parameter {
            name: "current".into(),
        },
        bp::Input::Literal {
            value_type: phase.clone(),
            value: serde_json::json!(1),
        },
    );
    let mut branch_node = node(&branch, bp::NodeKind::Branch);
    branch_node.inputs.insert("condition".into(), link(&opened));
    let mut nodes = vec![first, opened_node];
    deactivate_owner(DOOR, &mut nodes, &mut branch_node, "true");
    nodes.push(branch_node);
    for (i, n) in nodes.iter().enumerate() {
        asset
            .layout
            .positions
            .insert(n.id.clone(), [(i % 3) as f32 * 260., (i / 3) as f32 * 180.]);
    }
    asset.functions.push(bp::Graph {
        id: id(DOOR, "function.door_changed"),
        name: "door_changed".into(),
        override_id: Some(
            "cpp:c:@N@epok@S@DoorComponent@F@door_changed#$@N@epok@E@DoorState#S0_#".into(),
        ),
        timeline: None,
        parameters: vec![
            Parameter {
                name: "previous".into(),
                value_type: phase.clone(),
                direction: crate::reflection_schema::Direction::Value,
            },
            Parameter {
                name: "current".into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{CreateOptions, Template};

    #[test]
    fn generated_world_domain_rejects_narrowed_key_door_and_flag_roles() {
        let root = crate::workspace::tests::temp("dungeon-role-domains");
        crate::workspace::create(&root, "Domains", Template::Basic).unwrap();
        let (bindings, _) = ensure(&root).unwrap();
        let original = crate::template_install::read(&root).unwrap().unwrap();
        for (role, base) in [
            ("key", PICKUP_BASE),
            ("door", DOOR_BASE),
            ("key_state", FLAG_BASE),
        ] {
            let mut asset = bp::BlueprintAsset::new(format!("Only3D{role}"), base.into());
            asset.family = Some(ClassFamily::Component);
            asset.component = Some(ComponentContract {
                owners: [Domain::World3D].into_iter().collect(),
                ..Default::default()
            });
            if role != "key_state" {
                asset.variables.push(bp::Variable {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: LOCK_ID.into(),
                    value_type: Type::UInt32,
                    default: serde_json::json!(0),
                    editable: true,
                    timeline_animatable: false,
                });
            }
            let path = root.join(format!("assets/Blueprints/{}.epokbp", asset.name));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            crate::blueprint_asset::create(&path, &asset).unwrap();
            let mut roles = original.roles.clone();
            roles.insert(role.into(), ClassReference::new(&asset.name, &asset.id));
            crate::template_install::write(
                &root,
                Template::Basic,
                GameplayFlavor::Cpp,
                roles
                    .iter()
                    .map(|(role, class)| (role.as_str(), class.clone())),
            )
            .unwrap();
            let saved = std::fs::read(root.join(crate::template_install::PATH)).unwrap();
            let error = ensure_for_domain(&root, Domain::World2D).unwrap_err();
            assert!(
                error.contains(role) && error.contains("world domain"),
                "{error}"
            );
            assert_eq!(
                std::fs::read(root.join(crate::template_install::PATH)).unwrap(),
                saved
            );
            assert!(ensure_for_domain(&root, Domain::World3D).is_ok(), "{role}");
        }
        assert!(bindings.shared_key && bindings.shared_door);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejected_installation_restores_sources_settings_and_lua_identities() {
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("dungeon-install-rollback");
            crate::workspace::create_with_options(
                &root,
                "Rollback",
                CreateOptions::new(Template::Basic).with_gameplay(flavor),
            )
            .unwrap();
            let key = match flavor {
                GameplayFlavor::Cpp => CPP_FILES[0].0,
                GameplayFlavor::Lua => LUA_KEY_PATH,
                GameplayFlavor::Blueprint => BLUEPRINT_KEY_PATH,
            };
            let source = root.join(key);
            std::fs::create_dir_all(source.parent().unwrap()).unwrap();
            std::fs::write(&source, b"invalid authored source").unwrap();
            let settings = std::fs::read(root.join(crate::template_install::PATH)).unwrap();
            let registry = std::fs::read(root.join(crate::lua_identity::REGISTRY)).ok();
            assert!(ensure(&root).is_err(), "{flavor:?}");
            assert_eq!(std::fs::read(source).unwrap(), b"invalid authored source");
            assert_eq!(
                std::fs::read(root.join(crate::template_install::PATH)).unwrap(),
                settings
            );
            assert_eq!(
                std::fs::read(root.join(crate::lua_identity::REGISTRY)).ok(),
                registry
            );
            for path in CPP_FILES.iter().map(|(path, _)| *path).chain([
                LUA_KEY_PATH,
                LUA_DOOR_PATH,
                BLUEPRINT_KEY_PATH,
                BLUEPRINT_DOOR_PATH,
            ]) {
                assert_eq!(root.join(path).exists(), path == key, "{flavor:?}: {path}");
            }
            std::fs::remove_dir_all(root).unwrap();
        }
        let root = crate::workspace::tests::temp("dungeon-invalid-lua-registry");
        std::fs::create_dir_all(root.join("ProjectSettings")).unwrap();
        let path = root.join(crate::lua_identity::REGISTRY);
        std::fs::write(&path, b"invalid authored identity document").unwrap();
        assert!(install(&root, GameplayFlavor::Lua).is_err());
        assert_eq!(
            std::fs::read(path).unwrap(),
            b"invalid authored identity document"
        );
        assert!(!root.join(LUA_KEY_PATH).exists());
        assert!(!root.join(LUA_DOOR_PATH).exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_legacy_rule_sources_and_generated_bindings_are_preserved_in_every_flavor() {
        for flavor in GameplayFlavor::ALL {
            let parent = crate::workspace::tests::temp("legacy-dungeon-contract");
            let root = parent.join("Legacy");
            crate::workspace::create_with_options(
                &root,
                "Legacy",
                CreateOptions::new(Template::Basic).with_gameplay(flavor),
            )
            .unwrap();
            let files: &[(&str, &[u8])] = match flavor {
                GameplayFlavor::Cpp => &[
                    (
                        CPP_FILES[0].0,
                        include_bytes!("../tests/fixtures/dungeon-legacy/DungeonKey.hpp"),
                    ),
                    (
                        CPP_FILES[2].0,
                        include_bytes!("../tests/fixtures/dungeon-legacy/DungeonDoor.hpp"),
                    ),
                ],
                GameplayFlavor::Lua => &[
                    (
                        LUA_KEY_PATH,
                        include_bytes!("../tests/fixtures/dungeon-legacy/DungeonKey.lua"),
                    ),
                    (
                        LUA_DOOR_PATH,
                        include_bytes!("../tests/fixtures/dungeon-legacy/DungeonDoor.lua"),
                    ),
                ],
                GameplayFlavor::Blueprint => &[
                    (
                        BLUEPRINT_KEY_PATH,
                        include_bytes!("../tests/fixtures/dungeon-legacy/DungeonKey.epokbp"),
                    ),
                    (
                        BLUEPRINT_DOOR_PATH,
                        include_bytes!("../tests/fixtures/dungeon-legacy/DungeonDoor.epokbp"),
                    ),
                ],
            };
            for (path, bytes) in files {
                let target = root.join(path);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::write(target, bytes).unwrap();
            }
            let (bindings, written) = ensure(&root).unwrap();
            assert!(!bindings.shared_key && !bindings.shared_door);
            assert!(written.updated.is_empty());
            for (path, bytes) in files {
                assert_eq!(std::fs::read(root.join(path)).unwrap(), *bytes);
            }
            let mut scene = crate::scene::Scene::default();
            scene.actors.clear();
            let actor = crate::scene::Actor::cube("Legacy dungeon".into());
            let mut generator = crate::dungeon::Generator::new(actor.id);
            generator.source = crate::dungeon::Source::Recipe;
            generator.seed = 1;
            let id = generator.id;
            scene.actors.push(actor);
            scene.dungeons.push(generator);
            scene.refresh_actor_hierarchy();
            crate::dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
            assert!(
                scene.actors.iter().flat_map(|a| &a.components).all(|c| c
                    .class
                    .class_id
                    .as_deref()
                    != Some(FLAG_BASE))
            );
            let doors: Vec<_> = scene
                .actors
                .iter()
                .flat_map(|a| &a.components)
                .filter(|c| c.class.class_id == bindings.door.class_id)
                .collect();
            assert!(!doors.is_empty());
            for door in doors {
                let key = door.properties[KEY_REFERENCE].as_str().unwrap();
                assert!(scene.actors.iter().any(|a| a.id.to_string() == key));
                assert!(door.properties.contains_key(LOCK));
            }
            for (path, bytes) in files {
                assert_eq!(std::fs::read(root.join(path)).unwrap(), *bytes);
            }
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    #[test]
    fn recorded_dungeon_roles_resolve_renames_and_ignore_alternate_file_names() {
        let parent = crate::workspace::tests::temp("dungeon-recorded-roles");
        let root = parent.join("Roles");
        crate::workspace::create_with_options(&root, "Roles", CreateOptions::new(Template::Basic))
            .unwrap();
        let (original, _) = ensure(&root).unwrap();
        let saved = std::fs::read(root.join(crate::template_install::PATH)).unwrap();
        install(&root, GameplayFlavor::Blueprint).unwrap();
        for (path, name) in [
            (BLUEPRINT_KEY_PATH, "AlternateKey"),
            (BLUEPRINT_DOOR_PATH, "AlternateDoor"),
        ] {
            let mut asset = crate::blueprint_asset::load(&root.join(path)).unwrap();
            asset.name = name.into();
            std::fs::write(root.join(path), crate::document::to_vec(&asset).unwrap()).unwrap();
        }
        assert_eq!(installed(&root), Some(GameplayFlavor::Blueprint));
        for (path, old, new) in [
            (CPP_FILES[0].0, KEY, "RenamedKey"),
            (CPP_FILES[2].0, DOOR, "RenamedDoor"),
        ] {
            let source = std::fs::read_to_string(root.join(path)).unwrap();
            std::fs::write(root.join(path), source.replace(old, new)).unwrap();
        }
        let (resolved, written) = ensure(&root).unwrap();
        assert_eq!(resolved.key.class_id, original.key.class_id);
        assert_eq!(resolved.door.class_id, original.door.class_id);
        assert_eq!(resolved.key.name, "RenamedKey");
        assert_eq!(resolved.door.name, "RenamedDoor");
        assert_eq!(written, Written::default());
        assert_eq!(
            std::fs::read(root.join(crate::template_install::PATH)).unwrap(),
            saved
        );
        let mut installation = crate::template_install::read(&root).unwrap().unwrap();
        installation.roles.remove("door");
        let incomplete = crate::document::to_vec(&installation).unwrap();
        std::fs::write(root.join(crate::template_install::PATH), &incomplete).unwrap();
        assert!(ensure(&root).unwrap_err().contains("both key and door"));
        assert_eq!(
            std::fs::read(root.join(crate::template_install::PATH)).unwrap(),
            incomplete
        );
        installation
            .roles
            .insert("door".into(), original.door.clone());
        for (reference, diagnostic) in [
            (
                ClassReference::new("RenamedKey", &uuid::Uuid::new_v4().to_string()),
                "missing class UUID",
            ),
            (
                installation.roles["game_root"].clone(),
                "concrete world component",
            ),
        ] {
            installation.roles.insert("key".into(), reference);
            let rejected = crate::document::to_vec(&installation).unwrap();
            std::fs::write(root.join(crate::template_install::PATH), &rejected).unwrap();
            assert!(ensure(&root).unwrap_err().contains(diagnostic));
            assert_eq!(
                std::fs::read(root.join(crate::template_install::PATH)).unwrap(),
                rejected
            );
        }
        std::fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn older_installation_records_adopt_the_selected_flavor_without_replacing_roles() {
        for flavor in GameplayFlavor::ALL {
            let parent = crate::workspace::tests::temp("dungeon-role-adoption");
            let root = parent.join("Roles");
            crate::workspace::create_with_options(
                &root,
                "Roles",
                CreateOptions {
                    gameplay: flavor,
                    ..CreateOptions::new(Template::Basic)
                },
            )
            .unwrap();
            let before = crate::template_install::read(&root).unwrap().unwrap();
            let (resolved, _) = ensure(&root).unwrap();
            let expected: Bindings = classes(flavor).into();
            assert_eq!(resolved, expected);
            let after = crate::template_install::read(&root).unwrap().unwrap();
            assert_eq!(after.gameplay, flavor.key());
            assert_eq!(after.roles["game_root"], before.roles["game_root"]);
            assert_eq!(after.roles["key"], expected.key);
            assert_eq!(after.roles["door"], expected.door);
            assert_eq!(ensure(&root).unwrap().1, Written::default());
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

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
                (expected.key, &[LOCK_ID][..]),
                (expected.door, &[LOCK_ID][..]),
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
            let registry = crate::scripts::declaration_registry(&root).unwrap();
            let model = registry.model().unwrap();
            assert!(
                model
                    .class(expected.key.1)
                    .unwrap()
                    .ancestry
                    .iter()
                    .any(|id| id == PICKUP_BASE)
            );
            assert!(
                model
                    .class(expected.door.1)
                    .unwrap()
                    .ancestry
                    .iter()
                    .any(|id| id == DOOR_BASE)
            );
            match flavor {
                GameplayFlavor::Lua => {
                    let registry = crate::blueprint::native_registry(&root, &native).unwrap();
                    let files = crate::lua_asset::load_all(&root).unwrap();
                    for mode in [
                        crate::settings::LuaExecution::NativeCpp,
                        crate::settings::LuaExecution::VmBytecode,
                        crate::settings::LuaExecution::VmSource,
                    ] {
                        crate::lua_compile::compile(&root, &registry, &files, mode).unwrap_or_else(
                            |e| panic!("{mode:?}: {}", crate::lua_compile::report(&e)),
                        );
                    }
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
