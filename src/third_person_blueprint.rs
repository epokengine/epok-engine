//! The Blueprint flavor of the Third Person starter template.
//!
//! The same controller as `templates/ThirdPersonController.cpp` and
//! `templates/ThirdPersonController.lua`: camera-relative movement, gravity,
//! jumping, a step-up pass, a locomotion state machine and an obstacle-aware
//! orbit boom, authored as a visual class asset. It reaches the engine only
//! through reflected gameplay operations and Blueprint builtins, so a project
//! created with this flavor contains no project C++ at all.
//!
//! Everything here is constructed, never parsed: `asset()` is a pure function of
//! this source, so generating a project needs no reflection data and two calls
//! produce identical bytes.
//!
//! Two shapes a script takes for granted are missing from the node set, and the
//! template ships each as a function graph of its own rather than working
//! around it at every call site. A graph has no local variables and no
//! conditional value node, so `pick` is the missing Select, and `hold` and
//! `hold_flag` capture a value at a point in the execution order - pure data is
//! otherwise re-read where it is used, which would sample the position *after*
//! the move it has to be compared against. Each is one Branch or one Return
//! wide. `button_axis` is the third, and exists because the same two-button
//! axis is read twice.
use crate::blueprint_asset::{
    BinaryOp, BlueprintAsset, Builtin, Graph, Input, Node, NodeKind, Variable,
};
use crate::reflection_schema::{self as schema, Type};
use serde_json::json;
use std::collections::BTreeMap;

pub const CLASS_ID: &str = "a12673b4-b8e9-476d-8a49-bc50d69f5737";
pub const CLASS_NAME: &str = "ThirdPersonController";
pub const ASSET_PATH: &str = "assets/Blueprints/ThirdPersonController.epokbp";

/// Reflected gameplay operations, pinned by identity so a renumbered `Id=` in a
/// runtime header fails this template's own test instead of silently dropping a
/// node out of every generated project.
mod op {
    pub const CLAMP: &str = "6b6ba8bd-8446-4fef-a147-e41b61f88a61";
    pub const VECTOR3: &str = "4fe0f030-091c-4d55-984d-769077622c33";
    pub const SCALE: &str = "d32131a5-2c47-43f2-8907-35e40389cfb5";
    pub const SINE: &str = "eeee7d31-a8c6-437a-95db-10e163771b24";
    pub const COSINE: &str = "565552a8-869a-4ca7-a1d8-a7ad6e1508a7";
    pub const LENGTH2: &str = "57b625c6-06d7-4635-b551-17a75ceffa3b";
    pub const WRAP_DEGREES: &str = "9b882fef-4a7b-4463-81aa-65b2f7e0dcd0";
    pub const MOVE_TOWARD_DEGREES: &str = "bacfcfc1-e38c-44d2-b47d-19ec3ae37f8d";
    pub const HEADING_DEGREES: &str = "1057a007-0420-4b95-8f48-140be81f9ba5";
    pub const STICK_INTENT: &str = "5b0a28df-53d9-40e4-b7ca-c5381f473771";
    pub const INPUT_AXIS: &str = "c1100ee5-d446-4f34-a65f-4f05dfce1f5c";
    pub const COLLISION_MOVE: &str = "da298163-8f45-4b47-81c9-e2e70c89a566";
    pub const RAYCAST_SEGMENT: &str = "acebf3f3-9592-481f-820e-99e44b834f5d";
    pub const SET_CAMERA: &str = "e36ec48c-b2fe-49a2-8935-894d346882f7";
    pub const PLAY_CLIP: &str = "047cbafb-5024-44d8-92a3-4939b31de81d";
    pub const PAUSE_ANIMATION: &str = "74c0cd75-108d-486c-bccb-68b0a85b1a9a";
    pub const PLAYBACK_STATE: &str = "3ec24d35-4613-4ffb-b50a-0ea205e901b1";
    pub const CLIP_FRAMES: &str = "3ea752b4-6bf6-4e45-831c-b05524ecdc46";
    pub const CLIP_LOOP_TICKS: &str = "857927dd-9229-44d3-8556-b392f18f3567";
    pub const SET_ANIMATION_POSITION: &str = "b0b30f1a-a82b-4806-bb2a-eb7392bb6607";
}

/// `epok::ActorComponent::begin_play` and `::tick` carry no explicit `Id=`, so
/// their reflected identity is the Clang USR of the declaration. It is stable
/// across machines and checkouts; renaming either would need a migration anyway.
const BEGIN_PLAY: &str = "cpp:c:@N@epok@S@ActorComponent@F@begin_play#";
const TICK: &str = "cpp:c:@N@epok@S@ActorComponent@F@tick#$@N@psyqo@S@FixedPoint>#Vi12#I#Vi4096#";

/// Every collider layer: the controller is the player, so nothing is ignored.
const MASK: u64 = 4_294_967_295;

// Tuning. A class may declare sixteen properties in total and the scene
// bindings and the runtime state fill them, so these live as named literals
// here exactly as they do in the `constexpr` block of the C++ flavor. Every one
// is the same number the other two flavors use.
const MAX_SPEED: f64 = 6.0;
const WALK_CYCLE_SPEED: f64 = 2.6;
const RUN_CYCLE_SPEED: f64 = 6.0;
const TURN_RATE: f64 = 540.0;
const ORBIT_RATE: f64 = 140.0;
const PITCH_RATE: f64 = 70.0;
const PITCH_MINIMUM: f64 = -10.0;
const PITCH_MAXIMUM: f64 = 55.0;
const BOOM_LENGTH: f64 = 5.4;
const BOOM_HEIGHT: f64 = 1.1;
const GRAVITY: f64 = 24.0;
const JUMP_SPEED: f64 = 9.0;
const GROUND_BIAS: f64 = -0.05;
const STEP_HEIGHT: f64 = 0.6;
const GROUND_RESPONSE: f64 = 12.0;
const AIR_RESPONSE: f64 = 3.0;
const CAMERA_PULL_IN: f64 = 0.04;
const CAMERA_MINIMUM_BOOM: f64 = 0.12;
const START_PITCH: f64 = 18.0;
/// Q12-exact locomotion thresholds. See the Lua flavor for why each is spelled
/// out in full rather than rounded.
const IDLE_SPEED: f64 = 0.119_873_046_875;
const RUN_SPEED: f64 = 3.600_097_656_25;
const KEEP_RUNNING_SPEED: f64 = 3.0;
const MINIMUM_RATE: f64 = 0.2;
const MAXIMUM_RATE: f64 = 1.6;
const TICKS_PER_SECOND: f64 = 60.0;

/// Locomotion states and clip slots, as the numbers the three flavors share.
const IDLE: u64 = 0;
const WALK: u64 = 1;
const RUN: u64 = 2;
const JUMP_UP: u64 = 3;
const JUMP_DOWN: u64 = 4;
const LAND: u64 = 5;

const UP: u64 = 4;
const RIGHT: u64 = 5;
const DOWN: u64 = 6;
const LEFT: u64 = 7;
const CROSS: u64 = 14;
const LEFT_X: i64 = 0;
const LEFT_Y: i64 = 1;
const RIGHT_X: i64 = 2;
const RIGHT_Y: i64 = 3;
const PORT: u64 = 0;

/// Every identity in the asset is a digest of a stable authoring key, so the
/// generated file has no random content and regenerating it changes nothing.
fn ident(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(format!("epok:third-person-blueprint:{key}").as_bytes());
    let mut bytes: [u8; 16] = digest[..16].try_into().expect("digest covers a UUID");
    bytes[6] = (bytes[6] & 15) | 0x50;
    bytes[8] = (bytes[8] & 63) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}
fn variable_id(name: &str) -> String {
    ident(&format!("variable/{name}"))
}
fn graph_id(name: &str) -> String {
    ident(&format!("graph/{name}"))
}

fn actor3d() -> Type {
    Type::ActorRef {
        class: Some(crate::object_model::ACTOR3D_ID.into()),
    }
}
fn mesh3d() -> Type {
    Type::ComponentRef {
        class: Some(crate::object_model::MESH3D_COMPONENT_ID.into()),
    }
}
/// `epok::Axis`, written structurally because a saved graph stores the variants
/// it was authored against and is rechecked against the header on every build.
fn axis_type() -> Type {
    Type::Enum {
        cpp_name: "epok::Axis".into(),
        variants: [("LeftX", 0), ("LeftY", 1), ("RightX", 2), ("RightY", 3)]
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
    }
}

fn variables() -> Vec<Variable> {
    let make =
        |name: &str, value_type: Type, default: serde_json::Value, editable: bool| Variable {
            id: variable_id(name),
            name: name.into(),
            value_type,
            default,
            editable,
            timeline_animatable: false,
        };
    let mut out = vec![
        // Scene bindings, filled in by the template when the project is created,
        // so renaming an actor never breaks the controller.
        make("camera", actor3d(), json!(null), true),
        make("visual", mesh3d(), json!(null), true),
    ];
    for (name, value) in [
        ("idle_clip", IDLE),
        ("walk_clip", WALK),
        ("run_clip", RUN),
        ("jump_up_clip", JUMP_UP),
        ("jump_down_clip", JUMP_DOWN),
        ("land_clip", LAND),
    ] {
        out.push(make(name, Type::UInt32, json!(value), true));
    }
    // Runtime state. Hidden from the Inspector, still written by the graphs.
    out.push(make("camera_yaw", Type::Fixed, json!(0.0), false));
    out.push(make("camera_pitch", Type::Fixed, json!(START_PITCH), false));
    out.push(make("facing", Type::Fixed, json!(0.0), false));
    out.push(make(
        "velocity",
        Type::Vector { length: 3 },
        json!([0.0, 0.0, 0.0]),
        false,
    ));
    out.push(make("grounded", Type::Bool, json!(false), false));
    out.push(make("animation_state", Type::UInt32, json!(IDLE), false));
    out.push(make("playing_state", Type::UInt32, json!(LAND), false));
    out.push(make("playback_phase", Type::Fixed, json!(0.0), false));
    out
}

/// Where the value on a pin comes from.
#[derive(Clone)]
enum Src {
    /// An output pin of another node. A record result is read through a member
    /// path on the node's single `value` output, as `value.displacement.y`.
    Pin(String, String),
    Lit(Type, serde_json::Value),
    Param(String),
}
fn out(node: &str) -> Src {
    Src::Pin(node.into(), "value".into())
}
fn member(node: &str, path: &str) -> Src {
    Src::Pin(node.into(), format!("value.{path}"))
}
fn fixed(value: f64) -> Src {
    Src::Lit(Type::Fixed, json!(value))
}
fn count(value: u64) -> Src {
    Src::Lit(Type::UInt32, json!(value))
}
fn flag(value: bool) -> Src {
    Src::Lit(Type::Bool, json!(value))
}
fn axis(value: i64) -> Src {
    Src::Lit(axis_type(), json!(value))
}
fn arg(name: &str) -> Src {
    Src::Param(name.into())
}

/// A graph under construction.
///
/// Identities come from the graph name and a caller-chosen key, so the same
/// authored shape always writes the same file. Execution runs left to right
/// along a lane and the data feeding a step is stacked in the column above it,
/// which keeps the wires short and almost never crossing.
struct Build {
    graph: String,
    nodes: Vec<Node>,
    positions: BTreeMap<String, [f32; 2]>,
    comments: BTreeMap<String, String>,
    column: i32,
    row: i32,
    stacked: BTreeMap<i32, i32>,
}

impl Build {
    fn new(graph: &str) -> Self {
        Self {
            graph: graph.into(),
            nodes: vec![],
            positions: BTreeMap::new(),
            comments: BTreeMap::new(),
            column: 0,
            row: 0,
            stacked: BTreeMap::new(),
        }
    }
    fn place(&mut self, key: &str, kind: NodeKind, column: i32, row: i32) -> String {
        let id = ident(&format!("{}/{key}", self.graph));
        let slot = [column as f32 * 240., row as f32 * 150.];
        assert!(
            self.positions.insert(id.clone(), slot).is_none(),
            "duplicate node key {key} in {}",
            self.graph
        );
        self.nodes.push(Node {
            id: id.clone(),
            kind,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        });
        id
    }
    /// The next step of the current execution lane.
    fn step(&mut self, key: &str, kind: NodeKind) -> String {
        let (column, row) = (self.column, self.row);
        self.column += 1;
        self.place(key, kind, column, row)
    }
    /// Pure data feeding the step about to be created. A column holds six nodes
    /// before spilling to the left, so a deep expression stays legible.
    fn data(&mut self, key: &str, kind: NodeKind) -> String {
        let depth = self.stacked.entry(self.column).or_default();
        let (offset, slot) = (*depth / 6, *depth % 6);
        *depth += 1;
        self.place(key, kind, self.column - offset, -1 - slot)
    }
    /// Start a new execution lane, for a branch body or a sequence pin.
    fn lane(&mut self, column: i32, row: i32) {
        self.column = column;
        self.row = row;
    }
    fn node_mut(&mut self, id: &str) -> &mut Node {
        self.nodes
            .iter_mut()
            .find(|node| node.id == id)
            .expect("node belongs to this builder")
    }
    fn feed(&mut self, node: &str, pin: &str, src: Src) {
        let input = match src {
            Src::Pin(node, pin) => Input::Link { node, pin },
            Src::Lit(value_type, value) => Input::Literal { value_type, value },
            Src::Param(name) => Input::Parameter { name },
        };
        self.node_mut(node).inputs.insert(pin.into(), input);
    }
    fn exec(&mut self, from: &str, pin: &str, to: &str) {
        self.node_mut(from)
            .outputs
            .insert(pin.into(), vec![to.into()]);
    }
    fn chain(&mut self, steps: &[String]) {
        for pair in steps.windows(2) {
            self.exec(&pair[0], "next", &pair[1]);
        }
    }
    fn note(&mut self, node: &str, text: &str) {
        self.comments.insert(node.into(), text.into());
    }

    // Data nodes.
    fn read(&mut self, key: &str, variable: &str) -> String {
        self.data(
            key,
            NodeKind::GetVariable {
                member: variable_id(variable),
            },
        )
    }
    fn binary(&mut self, key: &str, operator: BinaryOp, a: Src, b: Src) -> String {
        let node = self.data(key, NodeKind::Binary { op: operator });
        self.feed(&node, "a", a);
        self.feed(&node, "b", b);
        node
    }
    fn negate(&mut self, key: &str, value: Src) -> String {
        self.binary(key, BinaryOp::Subtract, fixed(0.), value)
    }
    fn invert(&mut self, key: &str, value: Src) -> String {
        let node = self.data(key, NodeKind::Not);
        self.feed(&node, "value", value);
        node
    }
    fn component(&mut self, key: &str, value: Src, index: usize) -> String {
        let node = self.data(key, NodeKind::VectorComponent { length: 3, index });
        self.feed(&node, "value", value);
        node
    }
    fn vector(&mut self, key: &str, parts: [Src; 3]) -> String {
        let node = self.data(key, NodeKind::MakeVector { length: 3 });
        for (pin, value) in ["x", "y", "z"].into_iter().zip(parts) {
            self.feed(&node, pin, value);
        }
        node
    }
    /// A pure call on a static gameplay library.
    fn library(&mut self, key: &str, operation: &str, args: &[Src]) -> String {
        let node = self.data(
            key,
            NodeKind::Operation {
                operation: operation.into(),
            },
        );
        self.arguments(&node, operation, args);
        node
    }
    /// A pure call on a typed component reference.
    fn query(&mut self, key: &str, operation: &str, target: Src, args: &[Src]) -> String {
        let node = self.data(
            key,
            NodeKind::Operation {
                operation: operation.into(),
            },
        );
        self.feed(&node, "__target", target);
        self.arguments(&node, operation, args);
        node
    }
    fn adapter(&mut self, key: &str, operation: Builtin, pins: &[(&str, Src)]) -> String {
        let node = self.data(key, NodeKind::Builtin { operation });
        for (pin, value) in pins {
            self.feed(&node, pin, value.clone());
        }
        node
    }

    // Execution nodes.
    fn assign(&mut self, key: &str, variable: &str, value: Src) -> String {
        let node = self.step(
            key,
            NodeKind::SetVariable {
                member: variable_id(variable),
            },
        );
        self.feed(&node, "value", value);
        node
    }
    fn perform(&mut self, key: &str, operation: &str, args: &[Src]) -> String {
        let node = self.step(
            key,
            NodeKind::Operation {
                operation: operation.into(),
            },
        );
        self.arguments(&node, operation, args);
        node
    }
    fn perform_on(&mut self, key: &str, operation: &str, target: Src, args: &[Src]) -> String {
        let node = self.step(
            key,
            NodeKind::Operation {
                operation: operation.into(),
            },
        );
        self.feed(&node, "__target", target);
        self.arguments(&node, operation, args);
        node
    }
    fn drive(&mut self, key: &str, operation: Builtin, pins: &[(&str, Src)]) -> String {
        let node = self.step(key, NodeKind::Builtin { operation });
        for (pin, value) in pins {
            self.feed(&node, pin, value.clone());
        }
        node
    }
    fn call(&mut self, key: &str, graph: &str, args: &[(&str, Src)]) -> String {
        let node = self.step(
            key,
            NodeKind::Call {
                function: graph_id(graph),
            },
        );
        for (pin, value) in args {
            self.feed(&node, pin, value.clone());
        }
        node
    }
    fn branch(&mut self, key: &str, condition: Src) -> String {
        let node = self.step(key, NodeKind::Branch);
        self.feed(&node, "condition", condition);
        node
    }
    fn sequence(&mut self, key: &str) -> String {
        self.step(key, NodeKind::Sequence)
    }
    fn give(&mut self, key: &str, value: Src) -> String {
        let node = self.step(key, NodeKind::Return);
        self.feed(&node, "value", value);
        node
    }
    fn arguments(&mut self, node: &str, operation: &str, args: &[Src]) {
        for (index, value) in args.iter().enumerate() {
            self.feed(
                node,
                &format!("{operation}:parameter:{index}"),
                value.clone(),
            );
        }
    }

    fn finish(self, name: &str, parameters: Vec<schema::Parameter>, returns: Type) -> Built {
        Built {
            graph: Graph {
                id: graph_id(name),
                name: name.into(),
                override_id: match name {
                    "begin_play" => Some(BEGIN_PLAY.into()),
                    "tick" => Some(TICK.into()),
                    _ => None,
                },
                timeline: None,
                parameters,
                returns,
                entry: ident(&format!("{}/entry", self.graph)),
                nodes: self.nodes,
            },
            positions: self.positions,
            comments: self.comments,
        }
    }
}

struct Built {
    graph: Graph,
    positions: BTreeMap<String, [f32; 2]>,
    comments: BTreeMap<String, String>,
}

fn parameter(name: &str, value_type: Type) -> schema::Parameter {
    schema::Parameter {
        name: name.into(),
        value_type,
        direction: schema::Direction::Value,
    }
}
fn delta() -> Vec<schema::Parameter> {
    vec![parameter("delta_seconds", Type::Fixed)]
}

/// The authored controller graph. Stable: the same bytes every time.
pub fn asset() -> BlueprintAsset {
    let mut asset = BlueprintAsset::new(
        CLASS_NAME.into(),
        crate::object_model::ACTOR_COMPONENT_ID.into(),
    );
    asset.id = CLASS_ID.into();
    asset.family = Some(schema::ClassFamily::Component);
    // Narrowing the contract to World3D is what makes Get Owner resolve to an
    // Actor3D, which every spatial call in the graphs below needs.
    asset.component = Some(schema::ComponentContract {
        owners: [schema::Domain::World3D].into_iter().collect(),
        ..Default::default()
    });
    asset.variables = variables();
    for built in [
        begin_play(),
        tick(),
        orbit_camera(),
        move_body(),
        step_over(),
        animate(),
        choose_state(),
        clip_for(),
        clip_loops(),
        place_camera(),
        pick(),
        button_axis(),
        hold(),
        hold_flag(),
    ] {
        asset.layout.positions.extend(built.positions);
        asset.layout.comments.extend(built.comments);
        asset.functions.push(built.graph);
    }
    asset
}

/// The owning Actor3D of this component, and its transform.
fn owner(b: &mut Build) -> String {
    b.adapter("owner", Builtin::GetOwner, &[])
}
fn position(b: &mut Build, owner: &str) -> String {
    b.adapter("position", Builtin::GetPosition, &[("target", out(owner))])
}
fn rotation(b: &mut Build, owner: &str) -> String {
    b.adapter("rotation", Builtin::GetRotation, &[("target", out(owner))])
}

fn begin_play() -> Built {
    let mut b = Build::new("begin_play");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Reset the controller, take the camera over and place the boom.",
    );
    b.lane(1, 0);
    let owner = owner(&mut b);
    let rotation = rotation(&mut b, &owner);
    let yaw = b.component("owner_yaw", out(&rotation), 1);
    let mut steps = vec![entry, b.assign("facing", "facing", out(&yaw))];
    let facing = b.read("facing_again", "facing");
    steps.push(b.assign("camera_yaw", "camera_yaw", out(&facing)));
    steps.push(b.assign("camera_pitch", "camera_pitch", fixed(START_PITCH)));
    let rest = b.vector("rest", [fixed(0.0), fixed(0.0), fixed(0.0)]);
    steps.push(b.assign("velocity", "velocity", out(&rest)));
    steps.push(b.assign("grounded", "grounded", flag(false)));
    steps.push(b.assign("animation_state", "animation_state", count(IDLE)));
    // The scene authors the idle clip on the mesh, so the cycle starts on Land
    // and the first state change plays whatever the character is actually doing.
    steps.push(b.assign("playing_state", "playing_state", count(LAND)));
    steps.push(b.assign("playback_phase", "playback_phase", fixed(0.0)));
    let camera = b.read("camera", "camera");
    steps.push(b.perform("set_camera", op::SET_CAMERA, &[out(&camera)]));
    steps.push(b.call("place", "place_camera", &[]));
    b.chain(&steps);
    b.finish("begin_play", vec![], Type::Void)
}

fn tick() -> Built {
    let mut b = Build::new("tick");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(&entry, "Look, move, then follow with the camera.");
    b.lane(1, 0);
    let stalled = b.binary(
        "stalled",
        BinaryOp::LessEqual,
        arg("delta_seconds"),
        fixed(0.0),
    );
    let guard = b.branch("guard", out(&stalled));
    b.exec(&entry, "next", &guard);
    b.lane(2, 1);
    let steps = [
        b.call(
            "orbit",
            "orbit_camera",
            &[("delta_seconds", arg("delta_seconds"))],
        ),
        b.call(
            "move",
            "move_body",
            &[("delta_seconds", arg("delta_seconds"))],
        ),
        b.call("place", "place_camera", &[]),
    ];
    b.exec(&guard, "false", &steps[0]);
    b.chain(&steps);
    b.finish("tick", delta(), Type::Void)
}

fn orbit_camera() -> Built {
    let mut b = Build::new("orbit_camera");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Right stick look. Yaw wraps; pitch is clamped so the boom never passes \
         over the character's head or under the floor.",
    );
    b.lane(1, 0);
    let right_x = b.library("right_x", op::INPUT_AXIS, &[axis(RIGHT_X), count(PORT)]);
    let right_y = b.library("right_y", op::INPUT_AXIS, &[axis(RIGHT_Y), count(PORT)]);
    let look = b.library(
        "look",
        op::STICK_INTENT,
        &[member(&right_x, "value"), member(&right_y, "value")],
    );
    let turn = b.binary(
        "turn",
        BinaryOp::Multiply,
        member(&look, "x"),
        fixed(ORBIT_RATE),
    );
    let turn = b.binary(
        "turn_step",
        BinaryOp::Multiply,
        out(&turn),
        arg("delta_seconds"),
    );
    let yaw = b.read("camera_yaw", "camera_yaw");
    let yaw = b.binary("yaw_sum", BinaryOp::Add, out(&yaw), out(&turn));
    let yaw = b.library("yaw_wrapped", op::WRAP_DEGREES, &[out(&yaw)]);
    let set_yaw = b.assign("set_yaw", "camera_yaw", out(&yaw));

    let rise = b.binary(
        "rise",
        BinaryOp::Multiply,
        member(&look, "y"),
        fixed(PITCH_RATE),
    );
    let rise = b.binary(
        "rise_step",
        BinaryOp::Multiply,
        out(&rise),
        arg("delta_seconds"),
    );
    let pitch = b.read("camera_pitch", "camera_pitch");
    let pitch = b.binary("pitch_sum", BinaryOp::Subtract, out(&pitch), out(&rise));
    let pitch = b.library(
        "pitch_clamped",
        op::CLAMP,
        &[out(&pitch), fixed(PITCH_MINIMUM), fixed(PITCH_MAXIMUM)],
    );
    let set_pitch = b.assign("set_pitch", "camera_pitch", out(&pitch));
    b.chain(&[entry, set_yaw, set_pitch]);
    b.finish("orbit_camera", delta(), Type::Void)
}

fn move_body() -> Built {
    let mut b = Build::new("move_body");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Camera-relative movement, gravity, jumping and step handling.",
    );
    b.lane(1, 0);
    let owner = owner(&mut b);
    let position = position(&mut b, &owner);
    let velocity = b.read("velocity", "velocity");
    let velocity_x = b.component("velocity_x", out(&velocity), 0);
    let velocity_y = b.component("velocity_y", out(&velocity), 1);
    let velocity_z = b.component("velocity_z", out(&velocity), 2);
    let grounded = b.read("grounded", "grounded");

    // A jump belongs to the ground contact of this frame and `grounded` is
    // rewritten below, so the flag is captured before anything can change it.
    let pressed = b.adapter(
        "cross",
        Builtin::InputPressed,
        &[("button", count(CROSS)), ("port", count(PORT))],
    );
    let jumping = b.binary("jumping", BinaryOp::And, out(&grounded), out(&pressed));
    let jumped = b.call("jumped", "hold_flag", &[("value", out(&jumping))]);
    let airborne = b.invert("airborne", out(&jumped));

    let before_x = b.component("before_x_read", out(&position), 0);
    let before_x = b.call("before_x", "hold", &[("value", out(&before_x))]);
    let before_z = b.component("before_z_read", out(&position), 2);
    let before_z = b.call("before_z", "hold", &[("value", out(&before_z))]);

    // A held direction is full deflection, so the keyboard and the stick agree.
    let held = |b: &mut Build, key: &str, button: u64| {
        b.adapter(
            key,
            Builtin::InputHeld,
            &[("button", count(button)), ("port", count(PORT))],
        )
    };
    let right = held(&mut b, "held_right", RIGHT);
    let left = held(&mut b, "held_left", LEFT);
    let digital_x = b.call(
        "digital_x",
        "button_axis",
        &[("positive", out(&right)), ("negative", out(&left))],
    );
    let up = held(&mut b, "held_up", UP);
    let down = held(&mut b, "held_down", DOWN);
    let digital_z = b.call(
        "digital_z",
        "button_axis",
        &[("positive", out(&up)), ("negative", out(&down))],
    );
    let pushed_x = b.binary("pushed_x", BinaryOp::NotEqual, out(&digital_x), fixed(0.0));
    let pushed_z = b.binary("pushed_z", BinaryOp::NotEqual, out(&digital_z), fixed(0.0));
    let pushed = b.binary("pushed", BinaryOp::Or, out(&pushed_x), out(&pushed_z));
    let left_x = b.library("left_x", op::INPUT_AXIS, &[axis(LEFT_X), count(PORT)]);
    let move_x = b.call(
        "move_x",
        "pick",
        &[
            ("condition", out(&pushed)),
            ("when_true", out(&digital_x)),
            ("when_false", member(&left_x, "value")),
        ],
    );
    let left_y = b.library("left_y", op::INPUT_AXIS, &[axis(LEFT_Y), count(PORT)]);
    let move_z = b.call(
        "move_z",
        "pick",
        &[
            ("condition", out(&pushed)),
            ("when_true", out(&digital_z)),
            ("when_false", member(&left_y, "value")),
        ],
    );

    let intent = b.library("intent", op::STICK_INTENT, &[out(&move_x), out(&move_z)]);
    let yaw = b.read("camera_yaw", "camera_yaw");
    let yaw_sin = b.library("yaw_sin", op::SINE, &[out(&yaw)]);
    let yaw_cos = b.library("yaw_cos", op::COSINE, &[out(&yaw)]);
    let forward_x = b.binary(
        "forward_x",
        BinaryOp::Multiply,
        out(&yaw_sin),
        member(&intent, "y"),
    );
    let strafe_x = b.binary(
        "strafe_x",
        BinaryOp::Multiply,
        out(&yaw_cos),
        member(&intent, "x"),
    );
    let desired_x = b.binary(
        "desired_x_sum",
        BinaryOp::Add,
        out(&forward_x),
        out(&strafe_x),
    );
    let desired_x = b.binary(
        "desired_x",
        BinaryOp::Multiply,
        out(&desired_x),
        fixed(MAX_SPEED),
    );
    let forward_z = b.binary(
        "forward_z",
        BinaryOp::Multiply,
        out(&yaw_cos),
        member(&intent, "y"),
    );
    let strafe_z = b.binary(
        "strafe_z",
        BinaryOp::Multiply,
        out(&yaw_sin),
        member(&intent, "x"),
    );
    let desired_z = b.binary(
        "desired_z_sum",
        BinaryOp::Subtract,
        out(&forward_z),
        out(&strafe_z),
    );
    let desired_z = b.binary(
        "desired_z",
        BinaryOp::Multiply,
        out(&desired_z),
        fixed(MAX_SPEED),
    );

    let heading = b.library(
        "heading",
        op::HEADING_DEGREES,
        &[member(&intent, "x"), member(&intent, "y")],
    );
    let target = b.binary("target", BinaryOp::Add, out(&yaw), out(&heading));
    let turn_step = b.binary(
        "turn_step",
        BinaryOp::Multiply,
        fixed(TURN_RATE),
        arg("delta_seconds"),
    );
    let facing = b.read("facing", "facing");
    let turned = b.library(
        "turned",
        op::MOVE_TOWARD_DEGREES,
        &[out(&facing), out(&target), out(&turn_step)],
    );
    let pushing = b.binary(
        "pushing",
        BinaryOp::NotEqual,
        member(&intent, "strength"),
        fixed(0.0),
    );
    let next_facing = b.call(
        "next_facing",
        "pick",
        &[
            ("condition", out(&pushing)),
            ("when_true", out(&turned)),
            ("when_false", out(&facing)),
        ],
    );
    let set_facing = b.assign("set_facing", "facing", out(&next_facing));

    let response = b.call(
        "response",
        "pick",
        &[
            ("condition", out(&grounded)),
            ("when_true", fixed(GROUND_RESPONSE)),
            ("when_false", fixed(AIR_RESPONSE)),
        ],
    );
    // The response is never negative here, so clamping the low end as well as
    // the high end is exactly the reference `if response > 1 then response = 1`.
    let response_step = b.binary(
        "response_step",
        BinaryOp::Multiply,
        out(&response),
        arg("delta_seconds"),
    );
    let response_step = b.library(
        "response_clamped",
        op::CLAMP,
        &[out(&response_step), fixed(0.0), fixed(1.0)],
    );
    let gap_x = b.binary(
        "gap_x",
        BinaryOp::Subtract,
        out(&desired_x),
        out(&velocity_x),
    );
    let gap_x = b.binary(
        "gap_x_step",
        BinaryOp::Multiply,
        out(&gap_x),
        out(&response_step),
    );
    let next_x = b.binary("next_x", BinaryOp::Add, out(&velocity_x), out(&gap_x));
    let gap_z = b.binary(
        "gap_z",
        BinaryOp::Subtract,
        out(&desired_z),
        out(&velocity_z),
    );
    let gap_z = b.binary(
        "gap_z_step",
        BinaryOp::Multiply,
        out(&gap_z),
        out(&response_step),
    );
    let next_z = b.binary("next_z", BinaryOp::Add, out(&velocity_z), out(&gap_z));
    let planar = b.vector("planar", [out(&next_x), out(&velocity_y), out(&next_z)]);
    let set_planar = b.assign("set_planar", "velocity", out(&planar));

    let launch = b.call(
        "launch",
        "pick",
        &[
            ("condition", out(&jumped)),
            ("when_true", fixed(JUMP_SPEED)),
            ("when_false", fixed(0.0)),
        ],
    );
    let pull = b.binary(
        "pull",
        BinaryOp::Multiply,
        fixed(GRAVITY),
        arg("delta_seconds"),
    );
    let falling = b.binary("falling", BinaryOp::Subtract, out(&velocity_y), out(&pull));
    let next_y = b.call(
        "next_y",
        "pick",
        &[
            ("condition", out(&grounded)),
            ("when_true", out(&launch)),
            ("when_false", out(&falling)),
        ],
    );
    let vertical = b.vector(
        "vertical",
        [out(&velocity_x), out(&next_y), out(&velocity_z)],
    );
    let set_vertical = b.assign("set_vertical", "velocity", out(&vertical));

    // A small downward bias keeps a grounded character on ramps and steps.
    let settled = b.binary("settled", BinaryOp::And, out(&grounded), out(&airborne));
    let drop = b.binary(
        "drop",
        BinaryOp::Multiply,
        out(&velocity_y),
        arg("delta_seconds"),
    );
    let fall = b.call(
        "fall",
        "pick",
        &[
            ("condition", out(&settled)),
            ("when_true", fixed(GROUND_BIAS)),
            ("when_false", out(&drop)),
        ],
    );
    let stride_x = b.binary(
        "stride_x",
        BinaryOp::Multiply,
        out(&velocity_x),
        arg("delta_seconds"),
    );
    let stride_z = b.binary(
        "stride_z",
        BinaryOp::Multiply,
        out(&velocity_z),
        arg("delta_seconds"),
    );
    let movement = b.library(
        "movement",
        op::VECTOR3,
        &[out(&stride_x), out(&fall), out(&stride_z)],
    );
    let moved = b.perform(
        "moved",
        op::COLLISION_MOVE,
        &[out(&owner), out(&movement), count(MASK)],
    );

    let rising = b.binary("rising", BinaryOp::Greater, out(&velocity_y), fixed(0.0));
    let reached = b.binary(
        "reached",
        BinaryOp::Add,
        member(&moved, "displacement.y"),
        fixed(0.001),
    );
    let blocked_up = b.binary("blocked_up", BinaryOp::Less, out(&reached), out(&fall));
    let capped = b.binary("capped", BinaryOp::And, out(&rising), out(&blocked_up));
    let stopped_y = b.call(
        "stopped_y",
        "pick",
        &[
            ("condition", out(&capped)),
            ("when_true", fixed(0.0)),
            ("when_false", out(&velocity_y)),
        ],
    );
    let stopped = b.vector(
        "stopped",
        [out(&velocity_x), out(&stopped_y), out(&velocity_z)],
    );
    let clear_y = b.assign("clear_vertical", "velocity", out(&stopped));
    let join = b.sequence("join");
    b.chain(&[
        entry.clone(),
        jumped.clone(),
        before_x.clone(),
        before_z.clone(),
        digital_x,
        digital_z,
        move_x,
        move_z,
        next_facing,
        set_facing,
        response,
        set_planar,
        launch,
        next_y,
        set_vertical,
        fall.clone(),
        moved.clone(),
        stopped_y,
        clear_y,
        join.clone(),
    ]);

    // The ground contact of this frame is the sample the move just reported, so
    // the flag is written on both arms and read from the sample in the test.
    b.lane(2, 2);
    let blocked = b.binary(
        "blocked",
        BinaryOp::And,
        member(&moved, "blocked"),
        member(&moved, "grounded"),
    );
    let stepping = b.binary("stepping", BinaryOp::And, out(&blocked), out(&airborne));
    let step_branch = b.branch("step_branch", out(&stepping));
    b.exec(&join, "then_0", &step_branch);
    b.lane(3, 3);
    let land = b.assign("land", "grounded", member(&moved, "grounded"));
    let remaining_x = b.binary(
        "remaining_x",
        BinaryOp::Subtract,
        member(&movement, "x"),
        member(&moved, "displacement.x"),
    );
    let remaining_z = b.binary(
        "remaining_z",
        BinaryOp::Subtract,
        member(&movement, "z"),
        member(&moved, "displacement.z"),
    );
    let step = b.call(
        "step",
        "step_over",
        &[
            ("remaining_x", out(&remaining_x)),
            ("remaining_z", out(&remaining_z)),
        ],
    );
    b.exec(&step_branch, "true", &land);
    b.exec(&land, "next", &step);
    b.lane(3, 4);
    let stay = b.assign("stay", "grounded", member(&moved, "grounded"));
    b.exec(&step_branch, "false", &stay);

    b.lane(4, 1);
    let rotation = rotation(&mut b, &owner);
    let pitch = b.component("owner_pitch", out(&rotation), 0);
    let roll = b.component("owner_roll", out(&rotation), 2);
    let facing_now = b.read("facing_now", "facing");
    let heading = b.vector(
        "heading_vector",
        [out(&pitch), out(&facing_now), out(&roll)],
    );
    let face = b.drive(
        "face",
        Builtin::SetRotation,
        &[("target", out(&owner)), ("value", out(&heading))],
    );
    b.exec(&join, "then_1", &face);

    b.lane(5, 1);
    let now_x = b.component("now_x", out(&position), 0);
    let now_z = b.component("now_z", out(&position), 2);
    let travel_x = b.binary("travel_x", BinaryOp::Subtract, out(&now_x), out(&before_x));
    let travel_x = b.binary(
        "speed_x",
        BinaryOp::Divide,
        out(&travel_x),
        arg("delta_seconds"),
    );
    let travel_z = b.binary("travel_z", BinaryOp::Subtract, out(&now_z), out(&before_z));
    let travel_z = b.binary(
        "speed_z",
        BinaryOp::Divide,
        out(&travel_z),
        arg("delta_seconds"),
    );
    let speed = b.library("speed", op::LENGTH2, &[out(&travel_x), out(&travel_z)]);
    let animate = b.call(
        "animate",
        "animate",
        &[
            ("delta_seconds", arg("delta_seconds")),
            ("speed", out(&speed)),
            ("jumped", out(&jumped)),
        ],
    );
    b.exec(&join, "then_2", &animate);
    b.finish("move_body", delta(), Type::Void)
}

fn step_over() -> Built {
    let mut b = Build::new("step_over");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Lift, retry the blocked part of the step, then settle back down. A curb \
         or a ramp lip is walked over; a wall still stops the character.",
    );
    b.lane(1, 0);
    let still_x = b.binary("still_x", BinaryOp::Equal, arg("remaining_x"), fixed(0.0));
    let still_z = b.binary("still_z", BinaryOp::Equal, arg("remaining_z"), fixed(0.0));
    let still = b.binary("still", BinaryOp::And, out(&still_x), out(&still_z));
    let guard = b.branch("guard", out(&still));
    b.exec(&entry, "next", &guard);

    b.lane(2, 1);
    let owner = owner(&mut b);
    let up = b.library(
        "up",
        op::VECTOR3,
        &[fixed(0.0), fixed(STEP_HEIGHT), fixed(0.0)],
    );
    let lift = b.perform(
        "lift",
        op::COLLISION_MOVE,
        &[out(&owner), out(&up), count(MASK)],
    );
    let across = b.library(
        "across",
        op::VECTOR3,
        &[arg("remaining_x"), fixed(0.0), arg("remaining_z")],
    );
    let retry = b.perform(
        "retry",
        op::COLLISION_MOVE,
        &[out(&owner), out(&across), count(MASK)],
    );
    let back = b.binary(
        "back",
        BinaryOp::Subtract,
        fixed(GROUND_BIAS),
        member(&lift, "displacement.y"),
    );
    let down = b.library("down", op::VECTOR3, &[fixed(0.0), out(&back), fixed(0.0)]);
    let settle = b.perform(
        "settle",
        op::COLLISION_MOVE,
        &[out(&owner), out(&down), count(MASK)],
    );
    let land = b.assign("land", "grounded", member(&settle, "grounded"));
    b.exec(&guard, "false", &lift);
    b.chain(&[lift, retry, settle, land]);
    b.finish(
        "step_over",
        vec![
            parameter("remaining_x", Type::Fixed),
            parameter("remaining_z", Type::Fixed),
        ],
        Type::Void,
    )
}

fn animate() -> Built {
    let mut b = Build::new("animate");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Walk and Run advance with distance rather than with the clock, so the \
         feet never skate. `playing_state` always names the clip the animator \
         is on, so the clip index needs no field of its own.",
    );
    b.lane(1, 0);
    let visual = b.read("visual", "visual");
    let playing_state = b.read("playing_state", "playing_state");
    let playing = b.call("playing", "clip_for", &[("state", out(&playing_state))]);
    let frames = b.query("frames", op::CLIP_FRAMES, out(&visual), &[out(&playing)]);
    let playback = b.query("playback", op::PLAYBACK_STATE, out(&visual), &[]);
    let single = b.binary("single", BinaryOp::LessEqual, out(&frames), count(1));
    let last = b.binary("last", BinaryOp::Subtract, out(&frames), count(1));
    let last = b.binary("last_tick", BinaryOp::Multiply, out(&last), count(2));
    // The second half is only reached when the clip has frames to run out of,
    // exactly as the reference guards the subtraction with `frames > 1`.
    let played_out = b.binary(
        "played_out",
        BinaryOp::GreaterEqual,
        member(&playback, "ticks"),
        out(&last),
    );
    let finished = b.binary("finished", BinaryOp::Or, out(&single), out(&played_out));
    let next_state = b.call(
        "next_state",
        "choose_state",
        &[
            ("speed", arg("speed")),
            ("finished", out(&finished)),
            ("jumped", arg("jumped")),
        ],
    );
    let join = b.sequence("join");
    b.chain(&[entry, playing, next_state.clone(), join.clone()]);

    b.lane(2, 2);
    let changed = b.binary(
        "changed",
        BinaryOp::NotEqual,
        out(&next_state),
        out(&playing_state),
    );
    let change = b.branch("change", out(&changed));
    b.exec(&join, "then_0", &change);
    b.lane(3, 3);
    let clip = b.call("clip", "clip_for", &[("state", out(&next_state))]);
    let loops = b.call("loops", "clip_loops", &[("state", out(&next_state))]);
    let play = b.perform_on(
        "play",
        op::PLAY_CLIP,
        out(&visual),
        &[out(&clip), out(&loops)],
    );
    let accepted = b.branch("accepted", out(&play));
    b.exec(&change, "true", &clip);
    b.chain(&[clip, loops, play, accepted.clone()]);
    b.lane(7, 4);
    let adopt = b.assign("adopt", "playing_state", out(&next_state));
    let restart = b.assign("restart", "playback_phase", fixed(0.0));
    b.exec(&accepted, "true", &adopt);
    b.exec(&adopt, "next", &restart);

    b.lane(2, 6);
    let walking = b.binary("walking", BinaryOp::Equal, out(&next_state), count(WALK));
    let running = b.binary("running", BinaryOp::Equal, out(&next_state), count(RUN));
    let cycling = b.binary("cycling", BinaryOp::Or, out(&walking), out(&running));
    let cycle_branch = b.branch("cycle_branch", out(&cycling));
    b.exec(&join, "then_1", &cycle_branch);

    b.lane(3, 7);
    let pause = b.perform_on("pause", op::PAUSE_ANIMATION, out(&visual), &[]);
    let cycle = b.call(
        "cycle",
        "pick",
        &[
            ("condition", out(&walking)),
            ("when_true", fixed(WALK_CYCLE_SPEED)),
            ("when_false", fixed(RUN_CYCLE_SPEED)),
        ],
    );
    let rate = b.binary("rate", BinaryOp::Divide, arg("speed"), out(&cycle));
    let rate = b.library(
        "rate_clamped",
        op::CLAMP,
        &[out(&rate), fixed(MINIMUM_RATE), fixed(MAXIMUM_RATE)],
    );
    let advance = b.binary(
        "advance",
        BinaryOp::Multiply,
        out(&rate),
        arg("delta_seconds"),
    );
    let advance = b.binary(
        "advance_ticks",
        BinaryOp::Multiply,
        out(&advance),
        fixed(TICKS_PER_SECOND),
    );
    let phase = b.read("phase", "playback_phase");
    let phase_sum = b.binary("phase_sum", BinaryOp::Add, out(&phase), out(&advance));
    let set_phase = b.assign("set_phase", "playback_phase", out(&phase_sum));
    // The current clip again: a successful play above has already adopted the
    // new state, so this is the clip the animator is actually running.
    let current_state = b.read("current_state", "playing_state");
    let current = b.call("current", "clip_for", &[("state", out(&current_state))]);
    let wrap = b.sequence("wrap");
    b.exec(&cycle_branch, "true", &pause);
    b.chain(&[pause, cycle, set_phase, current.clone(), wrap.clone()]);

    // Wrapping the cycle needs no unbounded loop: the phase advances by at most
    // 1.6 ticks in a fixed step and the shortest clip loops over two, so four
    // guarded subtractions can never leave anything left to wrap.
    let period = b.query(
        "period",
        op::CLIP_LOOP_TICKS,
        out(&visual),
        &[out(&current)],
    );
    for pass in 0..4 {
        b.lane(9, 8 + pass * 2);
        let phase = b.read(&format!("phase_{pass}"), "playback_phase");
        let over = b.binary(
            &format!("over_{pass}"),
            BinaryOp::GreaterEqual,
            out(&phase),
            out(&period),
        );
        let branch = b.branch(&format!("wrap_{pass}"), out(&over));
        b.exec(&wrap, &format!("then_{pass}"), &branch);
        b.lane(10, 9 + pass * 2);
        let reduced = b.binary(
            &format!("reduced_{pass}"),
            BinaryOp::Subtract,
            out(&phase),
            out(&period),
        );
        let unwrap = b.assign(&format!("unwrap_{pass}"), "playback_phase", out(&reduced));
        b.exec(&branch, "true", &unwrap);
    }
    b.lane(9, 16);
    let final_phase = b.read("final_phase", "playback_phase");
    let cursor = b.perform_on(
        "cursor",
        op::SET_ANIMATION_POSITION,
        out(&visual),
        &[out(&final_phase)],
    );
    b.exec(&wrap, "then_4", &cursor);
    b.finish(
        "animate",
        vec![
            parameter("delta_seconds", Type::Fixed),
            parameter("speed", Type::Fixed),
            parameter("jumped", Type::Bool),
        ],
        Type::Void,
    )
}

fn choose_state() -> Built {
    let mut b = Build::new("choose_state");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "The locomotion state machine. 0 Idle 1 Walk 2 Run 3 JumpUp 4 JumpDown \
         5 Land; a steady jog keeps running below the speed it takes to start \
         one, so it does not flicker.",
    );
    b.lane(1, 0);
    let jumped = b.branch("jumped", arg("jumped"));
    b.exec(&entry, "next", &jumped);

    // Every leaf writes the state it reports, so the reads that select it still
    // see the state of the previous frame.
    let leaf = |b: &mut Build, key: &str, state: u64, column: i32, row: i32| {
        b.lane(column, row);
        let set = b.assign(&format!("set_{key}"), "animation_state", count(state));
        let give = b.give(&format!("give_{key}"), count(state));
        b.exec(&set, "next", &give);
        set
    };
    let jump_up = leaf(&mut b, "jump_up", JUMP_UP, 6, 1);
    let jump_down = leaf(&mut b, "jump_down", JUMP_DOWN, 6, 3);
    let land = leaf(&mut b, "land", LAND, 6, 5);
    let idle = leaf(&mut b, "idle", IDLE, 6, 7);
    let walk = leaf(&mut b, "walk", WALK, 6, 9);
    let run = leaf(&mut b, "run", RUN, 6, 11);
    b.exec(&jumped, "true", &jump_up);

    b.lane(2, 13);
    let grounded = b.read("grounded", "grounded");
    let airborne = b.invert("airborne", out(&grounded));
    let in_air = b.branch("in_air", out(&airborne));
    b.exec(&jumped, "false", &in_air);

    b.lane(3, 14);
    let velocity = b.read("velocity", "velocity");
    let velocity_y = b.component("velocity_y", out(&velocity), 1);
    let rising = b.binary("rising", BinaryOp::Greater, out(&velocity_y), fixed(0.0));
    let ascending = b.branch("ascending", out(&rising));
    b.exec(&in_air, "true", &ascending);
    b.exec(&ascending, "true", &jump_up);
    b.exec(&ascending, "false", &jump_down);

    b.lane(3, 15);
    let state = b.read("state", "animation_state");
    let was_up = b.binary("was_up", BinaryOp::Equal, out(&state), count(JUMP_UP));
    let was_down = b.binary("was_down", BinaryOp::Equal, out(&state), count(JUMP_DOWN));
    let was_air = b.binary("was_air", BinaryOp::Or, out(&was_up), out(&was_down));
    let landing = b.branch("landing", out(&was_air));
    b.exec(&in_air, "false", &landing);
    b.exec(&landing, "true", &land);

    b.lane(4, 16);
    let not_landing = b.binary("not_landing", BinaryOp::NotEqual, out(&state), count(LAND));
    let settled = b.binary("settled", BinaryOp::Or, out(&not_landing), arg("finished"));
    let reconsider = b.branch("reconsider", out(&settled));
    b.exec(&landing, "false", &reconsider);
    b.lane(5, 22);
    let unchanged = b.give("unchanged", out(&state));
    b.exec(&reconsider, "false", &unchanged);

    b.lane(5, 17);
    let slow = b.binary("slow", BinaryOp::Less, arg("speed"), fixed(IDLE_SPEED));
    let standing = b.branch("standing", out(&slow));
    b.exec(&reconsider, "true", &standing);
    b.exec(&standing, "true", &idle);
    b.lane(5, 19);
    let fast = b.binary("fast", BinaryOp::Greater, arg("speed"), fixed(RUN_SPEED));
    let sprinting = b.branch("sprinting", out(&fast));
    b.exec(&standing, "false", &sprinting);
    b.exec(&sprinting, "true", &run);
    b.lane(5, 21);
    let was_run = b.binary("was_run", BinaryOp::Equal, out(&state), count(RUN));
    let still_fast = b.binary(
        "still_fast",
        BinaryOp::Greater,
        arg("speed"),
        fixed(KEEP_RUNNING_SPEED),
    );
    let keeps_running = b.binary(
        "keeps_running",
        BinaryOp::And,
        out(&was_run),
        out(&still_fast),
    );
    let keeping = b.branch("keeping", out(&keeps_running));
    b.exec(&sprinting, "false", &keeping);
    b.exec(&keeping, "true", &run);
    b.exec(&keeping, "false", &walk);
    b.finish(
        "choose_state",
        vec![
            parameter("speed", Type::Fixed),
            parameter("finished", Type::Bool),
            parameter("jumped", Type::Bool),
        ],
        Type::UInt32,
    )
}

fn clip_for() -> Built {
    let mut b = Build::new("clip_for");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(&entry, "The clip index each locomotion state plays.");
    let mut previous = entry;
    let mut pin = "next";
    for (index, (state, variable)) in [
        (IDLE, "idle_clip"),
        (WALK, "walk_clip"),
        (RUN, "run_clip"),
        (JUMP_UP, "jump_up_clip"),
        (JUMP_DOWN, "jump_down_clip"),
    ]
    .into_iter()
    .enumerate()
    {
        let row = index as i32 * 2;
        b.lane(1, row);
        let matches = b.binary(
            &format!("is_{variable}"),
            BinaryOp::Equal,
            arg("state"),
            count(state),
        );
        let branch = b.branch(&format!("branch_{variable}"), out(&matches));
        b.exec(&previous, pin, &branch);
        b.lane(2, row + 1);
        let clip = b.read(variable, variable);
        let give = b.give(&format!("give_{variable}"), out(&clip));
        b.exec(&branch, "true", &give);
        previous = branch;
        pin = "false";
    }
    b.lane(1, 10);
    let land = b.read("land_clip", "land_clip");
    let give = b.give("give_land_clip", out(&land));
    b.exec(&previous, pin, &give);
    b.finish(
        "clip_for",
        vec![parameter("state", Type::UInt32)],
        Type::UInt32,
    )
}

fn clip_loops() -> Built {
    let mut b = Build::new("clip_loops");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(&entry, "Only the standing and locomotion clips cycle.");
    b.lane(1, 0);
    let idle = b.binary("is_idle", BinaryOp::Equal, arg("state"), count(IDLE));
    let walk = b.binary("is_walk", BinaryOp::Equal, arg("state"), count(WALK));
    let run = b.binary("is_run", BinaryOp::Equal, arg("state"), count(RUN));
    let looping = b.binary("idle_or_walk", BinaryOp::Or, out(&idle), out(&walk));
    let looping = b.binary("looping", BinaryOp::Or, out(&looping), out(&run));
    let give = b.give("give", out(&looping));
    b.exec(&entry, "next", &give);
    b.finish(
        "clip_loops",
        vec![parameter("state", Type::UInt32)],
        Type::Bool,
    )
}

fn place_camera() -> Built {
    let mut b = Build::new("place_camera");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Spring the camera onto a boom behind the character and pull it in when \
         the boom would pass through level geometry.",
    );
    b.lane(1, 0);
    let owner = owner(&mut b);
    let position = position(&mut b, &owner);
    let pitch = b.read("camera_pitch", "camera_pitch");
    let yaw = b.read("camera_yaw", "camera_yaw");
    let pitch_cos = b.library("pitch_cos", op::COSINE, &[out(&pitch)]);
    let pitch_sin = b.library("pitch_sin", op::SINE, &[out(&pitch)]);
    let reach = b.binary(
        "reach",
        BinaryOp::Multiply,
        fixed(BOOM_LENGTH),
        out(&pitch_cos),
    );
    let yaw_sin = b.library("yaw_sin", op::SINE, &[out(&yaw)]);
    let yaw_cos = b.library("yaw_cos", op::COSINE, &[out(&yaw)]);
    let px = b.component("px", out(&position), 0);
    let py = b.component("py", out(&position), 1);
    let pz = b.component("pz", out(&position), 2);
    let eye = b.binary("eye", BinaryOp::Add, out(&py), fixed(BOOM_HEIGHT));
    let origin = b.library("origin", op::VECTOR3, &[out(&px), out(&eye), out(&pz)]);
    let behind_x = b.binary("behind_x", BinaryOp::Multiply, out(&yaw_sin), out(&reach));
    let behind_x = b.negate("offset_x", out(&behind_x));
    let behind_z = b.binary("behind_z", BinaryOp::Multiply, out(&yaw_cos), out(&reach));
    let behind_z = b.negate("offset_z", out(&behind_z));
    let above = b.binary(
        "offset_y",
        BinaryOp::Multiply,
        fixed(BOOM_LENGTH),
        out(&pitch_sin),
    );
    let offset = b.library(
        "offset",
        op::VECTOR3,
        &[out(&behind_x), out(&above), out(&behind_z)],
    );
    let hit = b.perform(
        "hit",
        op::RAYCAST_SEGMENT,
        &[
            out(&origin),
            out(&offset),
            count(MASK),
            out(&owner),
            flag(false),
        ],
    );
    // Stop short of the surface so the near plane never clips it. A reported
    // fraction never exceeds one, so the upper bound here is inert.
    let pulled = b.binary(
        "pulled",
        BinaryOp::Subtract,
        member(&hit, "fraction"),
        fixed(CAMERA_PULL_IN),
    );
    let pulled = b.library(
        "pulled_clamped",
        op::CLAMP,
        &[out(&pulled), fixed(CAMERA_MINIMUM_BOOM), fixed(1.0)],
    );
    let fraction = b.call(
        "fraction",
        "pick",
        &[
            ("condition", member(&hit, "hit")),
            ("when_true", out(&pulled)),
            ("when_false", fixed(1.0)),
        ],
    );
    let boom = b.library("boom", op::SCALE, &[out(&offset), out(&fraction)]);
    let eye_x = b.binary(
        "eye_x",
        BinaryOp::Add,
        member(&origin, "x"),
        member(&boom, "x"),
    );
    let eye_y = b.binary(
        "eye_y",
        BinaryOp::Add,
        member(&origin, "y"),
        member(&boom, "y"),
    );
    let eye_z = b.binary(
        "eye_z",
        BinaryOp::Add,
        member(&origin, "z"),
        member(&boom, "z"),
    );
    let placement = b.vector("placement", [out(&eye_x), out(&eye_y), out(&eye_z)]);
    let camera = b.read("camera", "camera");
    let move_camera = b.drive(
        "move_camera",
        Builtin::SetPosition,
        &[("target", out(&camera)), ("value", out(&placement))],
    );
    let aim = b.vector("aim", [out(&pitch), out(&yaw), fixed(0.0)]);
    let turn_camera = b.drive(
        "turn_camera",
        Builtin::SetRotation,
        &[("target", out(&camera)), ("value", out(&aim))],
    );
    b.chain(&[entry, hit, fraction, move_camera, turn_camera]);
    b.finish("place_camera", vec![], Type::Void)
}

/// The Select a graph does not have. Both arms are ordinary data, so the caller
/// also keeps the value in the execution order it was computed in.
fn pick() -> Built {
    let mut b = Build::new("pick");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(&entry, "Conditional value. Both arms are evaluated.");
    b.lane(1, 0);
    let branch = b.branch("branch", arg("condition"));
    b.exec(&entry, "next", &branch);
    b.lane(2, 1);
    let yes = b.give("yes", arg("when_true"));
    b.lane(2, 2);
    let no = b.give("no", arg("when_false"));
    b.exec(&branch, "true", &yes);
    b.exec(&branch, "false", &no);
    b.finish(
        "pick",
        vec![
            parameter("condition", Type::Bool),
            parameter("when_true", Type::Fixed),
            parameter("when_false", Type::Fixed),
        ],
        Type::Fixed,
    )
}

/// A held direction is full deflection and opposite directions cancel, so the
/// keyboard and the stick reach the same movement intent.
fn button_axis() -> Built {
    let mut b = Build::new("button_axis");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(&entry, "Two buttons read as one -1..1 axis.");
    b.lane(1, 0);
    let forward = b.branch("forward", arg("positive"));
    b.exec(&entry, "next", &forward);
    b.lane(2, 1);
    let both = b.branch("both", arg("negative"));
    b.lane(2, 3);
    let back = b.branch("back", arg("negative"));
    b.exec(&forward, "true", &both);
    b.exec(&forward, "false", &back);
    b.lane(3, 2);
    let centre = b.give("centre", fixed(0.0));
    b.lane(3, 1);
    let positive = b.give("positive", fixed(1.0));
    b.lane(3, 3);
    let negative = b.give("negative", fixed(-1.0));
    b.exec(&both, "true", &centre);
    b.exec(&both, "false", &positive);
    b.exec(&back, "true", &negative);
    b.exec(&back, "false", &centre);
    b.finish(
        "button_axis",
        vec![
            parameter("positive", Type::Bool),
            parameter("negative", Type::Bool),
        ],
        Type::Fixed,
    )
}

/// Capture a value where it is read. Pure data is otherwise re-evaluated at
/// every use, which would sample the world after a later mutation.
fn hold() -> Built {
    let mut b = Build::new("hold");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Remember a value at this point in the execution order.",
    );
    b.lane(1, 0);
    let give = b.give("give", arg("value"));
    b.exec(&entry, "next", &give);
    b.finish("hold", vec![parameter("value", Type::Fixed)], Type::Fixed)
}

fn hold_flag() -> Built {
    let mut b = Build::new("hold_flag");
    let entry = b.place("entry", NodeKind::Entry, 0, 0);
    b.note(
        &entry,
        "Remember a flag at this point in the execution order.",
    );
    b.lane(1, 0);
    let give = b.give("give", arg("value"));
    b.exec(&entry, "next", &give);
    b.finish(
        "hold_flag",
        vec![parameter("value", Type::Bool)],
        Type::Bool,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_authored_graph_is_stable_and_complete() {
        let first = crate::document::to_vec(&asset()).unwrap();
        assert_eq!(first, crate::document::to_vec(&asset()).unwrap());
        let asset = asset();
        assert_eq!(asset.id, CLASS_ID);
        assert_eq!(asset.parent, crate::object_model::ACTOR_COMPONENT_ID);
        assert_eq!(asset.variables.len(), 16, "the runtime property budget");
        for graph in &asset.functions {
            let live: std::collections::BTreeSet<_> = graph
                .compilation_nodes()
                .map(|node| node.id.clone())
                .collect();
            for node in &graph.nodes {
                assert!(
                    live.contains(&node.id),
                    "{}: {} is unreachable",
                    graph.name,
                    node.id
                );
                assert!(
                    asset.layout.positions.contains_key(&node.id),
                    "{}: {} has no position",
                    graph.name,
                    node.id
                );
            }
        }
    }

    /// The Blueprint compiler is the only proof that the graph is valid: it
    /// resolves every operation against the reflected runtime and generates the
    /// class the project will build.
    #[test]
    fn the_template_compiles_in_a_generated_project() {
        let root = crate::workspace::tests::temp("third-person-blueprint");
        crate::workspace::create(&root, "BP Third Person", crate::workspace::Template::Basic)
            .unwrap();
        let path = crate::assets::inside(&root, ASSET_PATH).unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        crate::blueprint_asset::create(&path, &asset()).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap_or_else(|e| panic!("{e}"));
        let script = catalog
            .iter()
            .find(|script| script.name == CLASS_NAME)
            .unwrap_or_else(|| panic!("{:?}", catalog.iter().map(|s| &s.name).collect::<Vec<_>>()));
        // The scene writes its property overrides by name, so the published
        // class has to carry exactly the names the template binds.
        assert_eq!(
            script
                .properties
                .iter()
                .map(|property| property.name.as_str())
                .collect::<std::collections::BTreeSet<_>>(),
            asset()
                .variables
                .iter()
                .map(|variable| variable.name.as_str())
                .collect::<std::collections::BTreeSet<_>>()
        );
        // Every graph body reached code generation, not only the declaration.
        let native = crate::scripts::native_catalog(&root).unwrap();
        let compiled = crate::scripts::compile_blueprints(&root, &native)
            .unwrap()
            .expect("the template is the project's only Blueprint");
        let generated = compiled.artifacts.files
            [&std::path::PathBuf::from(format!("scripts/generated/{CLASS_ID}.hpp"))]
            .clone();
        let generated = String::from_utf8(generated).unwrap();
        for graph in &asset().functions {
            assert!(
                generated.contains(&format!("{}(", graph.name)),
                "{} is missing from the generated class",
                graph.name
            );
        }
        std::fs::remove_dir_all(&root).unwrap();
    }
}
