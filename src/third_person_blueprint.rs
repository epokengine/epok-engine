//! Project-owned starter input extension over the shared character mechanisms.
use crate::blueprint_asset::{BlueprintAsset, Graph, Input, Node, NodeKind};
use crate::reflection_schema::{self as schema, Type};
use std::collections::BTreeMap;

pub const CLASS_ID: &str = "a12673b4-b8e9-476d-8a49-bc50d69f5737";
pub const CLASS_NAME: &str = "ThirdPersonController";
pub const ASSET_PATH: &str = "assets/Blueprints/ThirdPersonController.epokbp";

// Keep the pre-extraction controller as an independently compiled test oracle.
// Production generation never installs it or its private state machine.
#[cfg(test)]
#[path = "third_person_blueprint_legacy.rs"]
mod legacy;
#[cfg(test)]
pub use legacy::asset;

fn identity(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(format!("epok:third-person-foundation:{key}").as_bytes());
    let mut bytes: [u8; 16] = digest[..16].try_into().expect("UUID digest");
    bytes[6] = (bytes[6] & 15) | 0x50;
    bytes[8] = (bytes[8] & 63) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}

pub fn shared_asset(registry: &crate::blueprint::Registry) -> Result<BlueprintAsset, String> {
    let input = registry
        .named("epok::LocalPlayer3DInputComponent")
        .ok_or("Shared third-person input class is unresolved.")?;
    let camera = input
        .properties
        .iter()
        .find(|p| p.name == "camera")
        .ok_or("Shared third-person camera binding is unresolved.")?;
    let rig = registry
        .named("epok::CameraRig3D")
        .ok_or("Shared camera rig class is unresolved.")?;
    let activate = rig
        .functions
        .iter()
        .find(|f| f.name == "make_active")
        .ok_or("Shared camera activation method is unresolved.")?;
    let mut asset = BlueprintAsset::new(CLASS_NAME.into(), input.id.clone());
    asset.id = CLASS_ID.into();
    asset.family = Some(schema::ClassFamily::Component);
    asset.component = input.component.clone();
    let entry = identity("entry");
    let parent = identity("parent");
    let read = identity("camera");
    let active = identity("activate");
    asset.functions.push(Graph {
        id: identity("begin_play"),
        name: "begin_play".into(),
        override_id: Some("cpp:c:@N@epok@S@ActorComponent@F@begin_play#".into()),
        timeline: None,
        parameters: vec![],
        returns: Type::Void,
        entry: entry.clone(),
        nodes: vec![
            Node {
                id: entry.clone(),
                kind: NodeKind::Entry,
                inputs: BTreeMap::new(),
                outputs: [("next".into(), vec![parent.clone()])].into(),
            },
            Node {
                id: parent.clone(),
                kind: NodeKind::CallParent,
                inputs: BTreeMap::new(),
                outputs: [("next".into(), vec![active.clone()])].into(),
            },
            Node {
                id: read.clone(),
                kind: NodeKind::GetVariable {
                    member: camera.id.clone(),
                },
                inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
            },
            Node {
                id: active.clone(),
                kind: NodeKind::Operation {
                    operation: activate.id.clone(),
                },
                inputs: [(
                    "__target".into(),
                    Input::Link {
                        node: read.clone(),
                        pin: "value".into(),
                    },
                )]
                .into(),
                outputs: BTreeMap::new(),
            },
        ],
    });
    asset.layout.positions = [
        (entry, [0., 0.]),
        (parent, [240., 0.]),
        (read, [240., 180.]),
        (active.clone(), [480., 0.]),
    ]
    .into();
    asset.layout.comments.insert(
        active,
        "Activate the typed camera binding. Extend calculate_command for project movement rules."
            .into(),
    );
    Ok(asset)
}
