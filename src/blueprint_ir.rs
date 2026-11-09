//! Typed, structured lowering. No graph is allowed to emit raw user C++ text.
use crate::{
    blueprint_asset::{BinaryOp, Builtin, Graph, Input, Node, NodeKind},
    reflection_schema::{self as schema, Type},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub value_type: Type,
    pub cpp: String,
    pub lvalue: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Node(String),
    Assign {
        target: String,
        value: Expression,
    },
    Evaluate(String),
    CheckOwner(Type),
    Delay(Expression),
    WaitPlayback {
        node: String,
        repeating: bool,
        target: Expression,
        asset: u64,
        marker: u64,
        reached: Vec<Statement>,
        completed: Vec<Statement>,
        cancelled: Vec<Statement>,
    },
    Timeline {
        node: String,
        target: String,
        keys: Vec<(String, String)>,
        looping: bool,
        updated: Vec<Statement>,
        finished: Vec<Statement>,
    },
    StopTimeline(String),
    Branch {
        condition: Expression,
        yes: Vec<Statement>,
        no: Vec<Statement>,
    },
    Loop {
        count: u32,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
}
/// Rejoin identical authored continuations after synchronous branches.
///
/// The validated IR declares captured call temporaries at function scope; its
/// expressions cannot introduce an authored C++ block-local declaration. We
/// move whole statement units only, starting at an identical authored Node ID.
/// This keeps trace/source attribution together with its effects and does not
/// merge unrelated getters, infer purity, or move a state read before a write.
/// Availability/dominance checks have already completed on the expanded graph.
/// Latent functions keep their existing state-machine scopes byte-for-byte.
pub(crate) fn factor_shared_branch_tails(body: Vec<Statement>) -> Vec<Statement> {
    fn latent(body: &[Statement]) -> bool {
        body.iter().any(|statement| match statement {
            Statement::Delay(_)
            | Statement::WaitPlayback { .. }
            | Statement::Timeline { .. }
            | Statement::StopTimeline(_) => true,
            Statement::Branch { yes, no, .. } => latent(yes) || latent(no),
            Statement::Loop { body, .. } => latent(body),
            _ => false,
        })
    }
    fn factor(body: Vec<Statement>) -> Vec<Statement> {
        let mut result = Vec::new();
        for statement in body {
            match statement {
                Statement::Branch { condition, yes, no } => {
                    let mut yes = factor(yes);
                    let mut no = factor(no);
                    let mut common = 0;
                    while common < yes.len().min(no.len())
                        && yes[yes.len() - common - 1] == no[no.len() - common - 1]
                    {
                        common += 1;
                    }
                    let begin = yes.len() - common;
                    // A matching trailing assignment/return alone can belong to
                    // different authored nodes. Do not detach it from its Node
                    // marker or change its debugger/source identity.
                    let boundary =
                        (begin..yes.len()).find(|index| matches!(yes[*index], Statement::Node(_)));
                    let tail = if let Some(boundary) = boundary {
                        let common = yes.len() - boundary;
                        no.truncate(no.len() - common);
                        yes.split_off(boundary)
                    } else {
                        Vec::new()
                    };
                    // Keep the branch even when its arms are empty: evaluating
                    // the condition remains at its authored execution point.
                    result.push(Statement::Branch { condition, yes, no });
                    result.extend(tail);
                }
                Statement::Loop { count, body } => {
                    result.push(Statement::Loop {
                        count,
                        body: factor(body),
                    });
                }
                statement => result.push(statement),
            }
        }
        result
    }
    if latent(&body) { body } else { factor(body) }
}

#[derive(Clone, Debug)]
pub struct FunctionIr {
    pub signature: schema::Function,
    pub body: Vec<Statement>,
    pub temporaries: BTreeMap<String, Type>,
}
#[derive(Clone, Debug)]
pub struct Error {
    pub node: Option<String>,
    pub message: String,
}
/// The concrete instance that executes a graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelfKind {
    #[default]
    Actor,
    Component,
}
impl SelfKind {
    pub fn handle(self) -> &'static str {
        "this->id()"
    }
    pub fn entity_pointer(self) -> &'static str {
        "epok::bp::object_data(this->id())"
    }
    pub fn transform(self) -> &'static str {
        match self {
            Self::Actor => "epok::bp::actor_transform(this)",
            Self::Component => "epok::bp::component_transform(this)",
        }
    }
    pub fn actor(self) -> &'static str {
        match self {
            Self::Actor => "this",
            Self::Component => "this->get_owner()",
        }
    }
}
pub struct Context<'a> {
    pub registry: &'a crate::blueprint::Registry,
    /// Resolved object model. Present only when the compilation involves a class
    /// outside the legacy Behaviour family; Behaviour-only projects never build it.
    pub model: Option<&'a crate::object_model::Model>,
    pub self_kind: SelfKind,
    pub writable: &'a BTreeSet<String>,
    pub properties: &'a BTreeMap<String, schema::Property>,
    pub functions: &'a BTreeMap<String, schema::Function>,
    pub parent_name: &'a str,
    pub self_class: &'a str,
    pub parent_function: Option<&'a schema::Function>,
    pub playback_timelines: &'a [(std::path::PathBuf, crate::timeline::TimelineAsset)],
    pub playback_effects: &'a [(std::path::PathBuf, crate::particle_effect::ParticleEffect)],
}

/// The `epok::bp::api` entry point a builtin adapts to, for the operations whose
/// lowering is a plain call. `None` for the operations that need more than their
/// operands (the spawn family, the class predicates and `SelfObject`).
///
/// The receiver expressions a builtin lowering may need besides its operands.
/// A Blueprint body, a native Lua body and a generated VM binding case name the
/// same object differently; everything else about the lowering is shared.
pub struct SelfReceiver<'a> {
    /// `ObjectId` of the object running the body.
    pub id: &'a str,
    /// Pointer to the object running the body, for the component adapters.
    pub object: &'a str,
    /// The actor the object belongs to.
    pub actor: &'a str,
}
impl<'a> SelfReceiver<'a> {
    /// A body compiled into the class itself, where the object is `this`.
    pub fn this(actor: &'a str) -> Self {
        Self {
            id: "this->id()",
            object: "this",
            actor,
        }
    }
}
pub fn builtin_api(operation: &Builtin) -> Option<&'static str> {
    Some(match operation {
        Builtin::IsValid => "valid",
        Builtin::GetPosition => "position",
        Builtin::GetPosition2D => "position_2d",
        Builtin::SetPosition2D => "set_position_2d",
        Builtin::GetRotation2D => "rotation_2d",
        Builtin::SetRotation2D => "set_rotation_2d",
        Builtin::GetScale2D => "scale_2d",
        Builtin::SetScale2D => "set_scale_2d",
        Builtin::GetRectPosition => "rect_position",
        Builtin::SetRectPosition => "set_rect_position",
        Builtin::GetRectSize => "rect_size",
        Builtin::SetRectSize => "set_rect_size",
        Builtin::GetRotation => "rotation",
        Builtin::GetScale => "scale",
        Builtin::SetPosition => "set_position",
        Builtin::SetRotation => "set_rotation",
        Builtin::SetScale => "set_scale",
        Builtin::InputHeld => "held",
        Builtin::InputPressed => "pressed",
        Builtin::InputReleased => "released",
        Builtin::RequestScene => "request_scene",
        Builtin::SetActive => "set_active",
        Builtin::DestroyActor => "destroy",
        Builtin::PlayAudio => "play_audio",
        Builtin::StopAudio => "stop_audio",
        Builtin::SetTexture => "set_texture",
        Builtin::SetAudioClip => "set_audio_clip",
        Builtin::PlayVideo => "play_video",
        Builtin::StopVideo => "stop_video",
        Builtin::IsVideoPlaying => "video_playing",
        Builtin::StartPocketStationApp => "pocketstation_start",
        Builtin::InstallPocketStationApp => "pocketstation_install",
        Builtin::PlaySequenceComponent => "play_sequence_component",
        Builtin::StopSequence => "stop_sequence",
        Builtin::PauseSequence => "pause_sequence",
        Builtin::ResumeSequence => "resume_sequence",
        Builtin::PlayEffectComponent => "play_effect_component",
        Builtin::StopEffect => "stop_effect",
        Builtin::BurstEffect => "burst_effect",
        Builtin::PauseEffect => "pause_effect",
        Builtin::ResumeEffect => "resume_effect",
        Builtin::EffectSequence => "effect_sequence",
        Builtin::GetTransform => "transform",
        Builtin::MakeTransform => "make_transform",
        _ => return None,
    })
}

/// The C++ expression a builtin lowers to, from already-lowered operands.
///
/// Both the Blueprint compiler and the Lua backends call this, so a builtin node
/// and its Lua spelling are the same generated call by construction rather than
/// by two implementations agreeing. `self_actor` is the expression naming the
/// actor the caller belongs to (`this`, or the component's owner).
pub fn builtin_cpp(
    operation: &Builtin,
    args: &[String],
    receiver: &SelfReceiver<'_>,
) -> Result<String, String> {
    let self_actor = receiver.actor;
    let arg = |index: usize| -> Result<&String, String> {
        args.get(index)
            .ok_or_else(|| format!("{operation:?} is missing operand {index}"))
    };
    Ok(match operation {
        Builtin::FindDataTableAsset { key, asset_kind } => {
            crate::data_table::validate_key(key)?;
            format!(
                "epok::bp::data_table_find({},{},{})",
                arg(0)?,
                serde_json::to_string(key).unwrap(),
                crate::data_table::kind_code(asset_kind)?
            )
        }
        Builtin::CastAsset { asset_kind } => format!(
            "epok::bp::data_table_cast({},{})",
            arg(0)?,
            crate::data_table::kind_code(asset_kind)?
        ),
        Builtin::SelfObject => receiver.id.into(),
        Builtin::GetOwner => format!("epok::bp::component_owner_id({})", receiver.object),
        Builtin::SpawnActor { .. } => format!(
            "epok::bp::spawn_actor({self_actor},{},{})",
            arg(0)?,
            arg(1)?
        ),
        Builtin::Spawn { class } => format!(
            "epok::bp::spawn_actor({self_actor},{}ULL,{})",
            crate::blueprint_refs::compact_id(class),
            arg(0)?
        ),
        Builtin::IsA { class } => format!(
            "epok::bp::is_a({},{}ULL)",
            arg(0)?,
            crate::blueprint_refs::compact_id(class)
        ),
        Builtin::Cast { class } => format!(
            "epok::bp::api::cast({}ULL,{})",
            crate::blueprint_refs::compact_id(class),
            arg(0)?
        ),
        Builtin::SpawnClass { base } => format!(
            "(epok::bp::class_is_a({}, {base}ULL)?epok::bp::spawn_actor({self_actor},{},{}):epok::ObjectId{{}})",
            arg(0)?,
            arg(0)?,
            arg(1)?,
            base = crate::blueprint_refs::compact_id(base)
        ),
        other => {
            let name = builtin_api(other)
                .ok_or_else(|| format!("{other:?} has no adapter entry point"))?;
            format!("epok::bp::api::{name}({})", args.join(","))
        }
    })
}

pub fn builtin_signature(operation: &Builtin) -> (Vec<(String, Type)>, Type, bool) {
    let entity = Type::ObjectRef { class: None };
    let spatial = Type::ActorRef {
        class: Some(crate::object_model::ACTOR3D_ID.into()),
    };
    let vector = Type::Vector { length: 3 };
    let (inputs, output, pure) = match operation {
        Builtin::FindDataTableAsset { asset_kind, .. } => (
            vec![(
                "table",
                Type::AssetRef {
                    kind: "DataTable".into(),
                },
            )],
            Type::AssetRef {
                kind: asset_kind.clone(),
            },
            true,
        ),
        Builtin::CastAsset { asset_kind } => (
            vec![(
                "asset",
                Type::AssetRef {
                    kind: "Asset".into(),
                },
            )],
            Type::AssetRef {
                kind: asset_kind.clone(),
            },
            true,
        ),
        Builtin::GetPosition2D => (
            vec![(
                "target",
                Type::ActorRef {
                    class: Some(crate::object_model::ACTOR2D_ID.into()),
                },
            )],
            Type::Vector { length: 2 },
            true,
        ),
        Builtin::SetPosition2D => (
            vec![
                (
                    "target",
                    Type::ActorRef {
                        class: Some(crate::object_model::ACTOR2D_ID.into()),
                    },
                ),
                ("value", Type::Vector { length: 2 }),
            ],
            Type::Void,
            false,
        ),
        Builtin::GetRotation2D => (
            vec![(
                "target",
                Type::ActorRef {
                    class: Some(crate::object_model::ACTOR2D_ID.into()),
                },
            )],
            Type::Fixed,
            true,
        ),
        Builtin::SetRotation2D => (
            vec![
                (
                    "target",
                    Type::ActorRef {
                        class: Some(crate::object_model::ACTOR2D_ID.into()),
                    },
                ),
                ("value", Type::Fixed),
            ],
            Type::Void,
            false,
        ),
        Builtin::GetScale2D => (
            vec![(
                "target",
                Type::ActorRef {
                    class: Some(crate::object_model::ACTOR2D_ID.into()),
                },
            )],
            Type::Vector { length: 2 },
            true,
        ),
        Builtin::SetScale2D => (
            vec![
                (
                    "target",
                    Type::ActorRef {
                        class: Some(crate::object_model::ACTOR2D_ID.into()),
                    },
                ),
                ("value", Type::Vector { length: 2 }),
            ],
            Type::Void,
            false,
        ),
        Builtin::GetRectPosition => (
            vec![(
                "target",
                Type::ActorRef {
                    class: Some(crate::object_model::UI_ACTOR_ID.into()),
                },
            )],
            Type::Vector { length: 2 },
            true,
        ),
        Builtin::SetRectPosition => (
            vec![
                (
                    "target",
                    Type::ActorRef {
                        class: Some(crate::object_model::UI_ACTOR_ID.into()),
                    },
                ),
                ("value", Type::Vector { length: 2 }),
            ],
            Type::Void,
            false,
        ),
        Builtin::GetRectSize => (
            vec![(
                "target",
                Type::ActorRef {
                    class: Some(crate::object_model::UI_ACTOR_ID.into()),
                },
            )],
            Type::Vector { length: 2 },
            true,
        ),
        Builtin::SetRectSize => (
            vec![
                (
                    "target",
                    Type::ActorRef {
                        class: Some(crate::object_model::UI_ACTOR_ID.into()),
                    },
                ),
                ("value", Type::Vector { length: 2 }),
            ],
            Type::Void,
            false,
        ),

        Builtin::PlayTimelineAsset { .. } => (vec![("owner", entity)], Type::SequenceHandle, false),
        Builtin::SpawnParticleEffect { .. } => (
            vec![
                (
                    "transform",
                    Type::Record {
                        cpp_name: "epok::Transform".into(),
                        fields: vec![],
                    },
                ),
                ("seed", Type::UInt32),
                ("owner", entity),
            ],
            Type::EffectHandle,
            false,
        ),
        Builtin::GetTransform => (
            vec![("target", spatial.clone())],
            Type::Record {
                cpp_name: "epok::Transform".into(),
                fields: vec![],
            },
            true,
        ),
        Builtin::MakeTransform => (
            vec![
                ("position", vector.clone()),
                ("rotation", vector.clone()),
                ("scale", vector),
            ],
            Type::Record {
                cpp_name: "epok::Transform".into(),
                fields: vec![],
            },
            true,
        ),
        Builtin::PlaySequenceComponent => (vec![("target", entity)], Type::SequenceHandle, false),
        Builtin::PlayEffectComponent => (vec![("target", entity)], Type::EffectHandle, false),
        Builtin::BurstEffect => (
            vec![("playback", Type::EffectHandle), ("count", Type::UInt32)],
            Type::Bool,
            false,
        ),
        Builtin::StopSequence | Builtin::PauseSequence | Builtin::ResumeSequence => {
            (vec![("playback", Type::SequenceHandle)], Type::Bool, false)
        }
        Builtin::StopEffect | Builtin::PauseEffect | Builtin::ResumeEffect => {
            (vec![("playback", Type::EffectHandle)], Type::Bool, false)
        }
        Builtin::EffectSequence => (
            vec![("playback", Type::EffectHandle)],
            Type::SequenceHandle,
            true,
        ),
        Builtin::SelfObject => (vec![], entity, true),
        Builtin::IsValid | Builtin::IsA { .. } => (vec![("target", entity)], Type::Bool, true),
        Builtin::Cast { class } => (
            vec![("target", entity)],
            Type::ObjectRef {
                class: Some(class.clone()),
            },
            true,
        ),
        Builtin::GetPosition | Builtin::GetRotation | Builtin::GetScale => {
            (vec![("target", spatial.clone())], vector, true)
        }
        Builtin::SetPosition | Builtin::SetRotation | Builtin::SetScale => (
            vec![("target", spatial.clone()), ("value", vector)],
            Type::Void,
            false,
        ),
        Builtin::InputHeld | Builtin::InputPressed | Builtin::InputReleased => (
            vec![("button", Type::UInt32), ("port", Type::UInt32)],
            Type::Bool,
            true,
        ),
        Builtin::RequestScene => (vec![("index", Type::UInt32)], Type::Bool, false),
        Builtin::SetActive => (
            vec![("target", entity), ("active", Type::Bool)],
            Type::Void,
            false,
        ),
        Builtin::DestroyActor | Builtin::PlayAudio | Builtin::StopAudio => {
            (vec![("target", entity)], Type::Void, false)
        }
        Builtin::SetTexture => (
            vec![
                ("target", entity),
                (
                    "texture",
                    Type::AssetRef {
                        kind: "Texture".into(),
                    },
                ),
            ],
            Type::Void,
            false,
        ),
        Builtin::SetAudioClip => (
            vec![
                ("target", entity),
                (
                    "clip",
                    Type::AssetRef {
                        kind: "AudioClip".into(),
                    },
                ),
            ],
            Type::Void,
            false,
        ),
        // Gameplay pauses while a video plays; the call queues it and returns
        // whether the clip exists and the CD can stream it.
        Builtin::PlayVideo => (
            vec![
                (
                    "video",
                    Type::AssetRef {
                        kind: "Video".into(),
                    },
                ),
                ("skippable", Type::Bool),
            ],
            Type::Bool,
            false,
        ),
        Builtin::StopVideo => (vec![], Type::Void, false),
        Builtin::IsVideoPlaying => (vec![], Type::Bool, true),
        // Installs the app on the PocketStation in `port` when it is missing
        // (or replaces it), then starts it; returns whether it succeeded.
        Builtin::StartPocketStationApp | Builtin::InstallPocketStationApp => (
            vec![
                (
                    "app",
                    Type::AssetRef {
                        kind: "PocketStationApp".into(),
                    },
                ),
                ("port", Type::UInt32),
            ],
            Type::Bool,
            false,
        ),
        Builtin::Spawn { class } => (
            vec![("parent", entity)],
            Type::ObjectRef {
                class: Some(class.clone()),
            },
            false,
        ),
        Builtin::GetOwner => (vec![], Type::ActorRef { class: None }, true),
        Builtin::SpawnActor { base } => (
            vec![
                ("class", Type::ClassRef { base: base.clone() }),
                ("parent", Type::ActorRef { class: None }),
            ],
            Type::ActorRef {
                class: Some(base.clone()),
            },
            false,
        ),
        Builtin::SpawnClass { base } => (
            vec![
                ("class", Type::ClassRef { base: base.clone() }),
                ("parent", entity),
            ],
            Type::ObjectRef {
                class: Some(base.clone()),
            },
            false,
        ),
    };
    (
        inputs
            .into_iter()
            .map(|(name, ty)| (name.into(), ty))
            .collect(),
        output,
        pure,
    )
}
pub fn assignable(actual: &Type, expected: &Type, registry: &crate::blueprint::Registry) -> bool {
    actual == expected
        || match (actual, expected) {
            // `Type::Enum` is structural and a graph stores the variants it saw when
            // the literal was authored, so a reflected enum that gains an enumerator
            // would otherwise stop matching its own pin in every saved graph. Accept
            // the same enum when every stored name still carries the same number:
            // the stored value was validated against the stored variants, so it is a
            // value that still exists. A renumbered or removed enumerator still
            // fails, which is what the compatibility rule actually protects.
            (
                Type::Enum {
                    cpp_name: a,
                    variants: stored,
                },
                Type::Enum {
                    cpp_name: b,
                    variants: current,
                },
            ) if a == b => stored
                .iter()
                .all(|(name, value)| current.get(name) == Some(value)),
            // The same append-only rule for registered records, which a graph stores
            // structurally whenever a record pin was promoted to a variable. Both
            // directions are accepted because the C++ type is the same object either
            // way and only the description is older; a reordered or retyped member
            // still fails. The empty field list is a nominal marker used elsewhere
            // and is deliberately left out of this rule.
            (
                Type::Record {
                    cpp_name: a,
                    fields: stored,
                },
                Type::Record {
                    cpp_name: b,
                    fields: current,
                },
            ) if a == b && !stored.is_empty() && !current.is_empty() => {
                let shared = stored.len().min(current.len());
                stored[..shared] == current[..shared]
            }
            (Type::AssetRef { .. }, Type::AssetRef { kind }) if kind == "Asset" => true,
            (Type::AssetRef { kind: a }, Type::AssetRef { kind: b })
                if matches!(b.as_str(), "AudioClip" | "PlayableAudio")
                    && matches!(a.as_str(), "AudioClip" | "PlayableAudio" | "MusicSequence") =>
            {
                true
            }
            (Type::ObjectRef { class: Some(a) }, Type::ObjectRef { class: Some(b) }) => {
                crate::blueprint_refs::class_is_a(registry, a, b)
            }
            (Type::ObjectRef { .. }, Type::ObjectRef { class: None }) => true,
            (Type::ActorRef { class: Some(a) }, Type::ActorRef { class: Some(b) })
            | (Type::ComponentRef { class: Some(a) }, Type::ComponentRef { class: Some(b) }) => {
                crate::blueprint_refs::class_is_a(registry, a, b)
            }
            (Type::ActorRef { .. }, Type::ActorRef { class: None })
            | (Type::ComponentRef { .. }, Type::ComponentRef { class: None })
            | (
                Type::ActorRef { .. } | Type::ComponentRef { .. },
                Type::ObjectRef { class: None },
            ) => true,
            (
                Type::ActorRef { class: Some(a) } | Type::ComponentRef { class: Some(a) },
                Type::ObjectRef { class: Some(b) },
            ) => crate::blueprint_refs::class_is_a(registry, a, b),
            (Type::ClassRef { base: a }, Type::ClassRef { base: b }) => {
                crate::blueprint_refs::class_is_a(registry, a, b)
            }
            _ => false,
        }
}
fn err(node: &str, message: impl Into<String>) -> Error {
    Error {
        node: Some(node.into()),
        message: message.into(),
    }
}
pub fn cpp_type(ty: &Type) -> Result<String, String> {
    Ok(match ty {
        Type::Void => "void".into(),
        Type::Bool => "bool".into(),
        Type::Int32 => "int32_t".into(),
        Type::UInt32 => "uint32_t".into(),
        Type::Fixed => "epok::Fixed".into(),
        Type::EffectLayerRef { .. } => "epok::EffectLayerHandle".into(),
        Type::SequenceHandle => "epok::timeline::Handle".into(),
        Type::EffectHandle => "epok::effects::Handle".into(),
        Type::AssetRef { .. } | Type::ClassRef { .. } => "uint64_t".into(),
        // Typed object references are compact generational identities, never raw
        // pointers: a stale id fails the generation check in ObjectRegistry::resolve
        // instead of reaching freed storage. All three families share one ABI word;
        // the static class of the pin is what distinguishes them at authoring time.
        Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. } => {
            "epok::ObjectId".into()
        }
        Type::Enum { cpp_name, .. } | Type::Record { cpp_name, .. } => {
            if !crate::scripts::class_identifier(cpp_name) {
                return Err("Invalid reflected C++ type name".into());
            }
            cpp_name.clone()
        }
        Type::Vector { length } if *length == 2 || *length == 3 => {
            format!("epok::bp::Vector<{length}>")
        }
        Type::Vector { .. } => return Err("Only bounded Vector2/Vector3 values are public".into()),
    })
}
pub fn literal(value: &serde_json::Value, ty: &Type) -> Result<String, String> {
    if !crate::script_values::valid(value, ty) {
        return Err(format!("Literal {value} is not {}", ty.label()));
    }
    match ty {
        Type::Record { .. } if !ty.members().is_empty() => {
            let fields = ty
                .members()
                .iter()
                .map(|field| {
                    literal(&value[&field.name], &field.value_type).map(|cpp| Expression {
                        value_type: field.value_type.clone(),
                        cpp,
                        lvalue: false,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            compose(ty, &fields)
        }
        Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
            if value.is_null() =>
        {
            Ok("epok::ObjectId{}".into())
        }
        Type::SequenceHandle => Ok("epok::timeline::Handle{}".into()),
        Type::EffectHandle => Ok("epok::effects::Handle{}".into()),
        Type::AssetRef { .. } | Type::ClassRef { .. } => {
            if value.is_null() {
                Ok("uint64_t(0)".into())
            } else {
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(value.as_str().unwrap().as_bytes());
                Ok(format!(
                    "{}ULL",
                    u64::from_le_bytes(hash[..8].try_into().unwrap())
                ))
            }
        }
        Type::Bool | Type::Int32 => Ok(value.to_string()),
        Type::UInt32 => Ok(format!("{value}u")),
        Type::Fixed => Ok(format!(
            "epok::Fixed({},epok::Fixed::RAW)",
            (value.as_f64().unwrap() * 4096.).round() as i32
        )),
        Type::Enum { cpp_name, .. } => {
            cpp_type(ty)?;
            Ok(format!("static_cast<{cpp_name}>({value})"))
        }
        Type::Vector { length } if *length == 2 || *length == 3 => Ok(format!(
            "epok::bp::Vector<{length}>{{{{{}}}}}",
            (0..*length)
                .map(|i| literal(&value[i], &Type::Fixed))
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        _ => Err("No literal adapter for this type".into()),
    }
}

fn compose(ty: &Type, values: &[Expression]) -> Result<String, String> {
    let fields = ty.members();
    if fields.is_empty() || fields.len() != values.len() {
        return Err("Invalid split pin type".into());
    }
    for (field, value) in fields.iter().zip(values) {
        if field.value_type != value.value_type || !crate::scripts::identifier(&field.name) {
            return Err(format!(
                "Split field {} has an incompatible type or name",
                field.name
            ));
        }
    }
    if let Type::Vector { length } = ty {
        return Ok(format!(
            "epok::bp::Vector<{length}>{{{{{}}}}}",
            values
                .iter()
                .map(|v| v.cpp.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    let mut cpp = format!("([&](){{ {} epok_split_value{{}};", cpp_type(ty)?);
    for (index, (field, value)) in fields.iter().zip(values).enumerate() {
        if let Type::Vector { length } = &field.value_type {
            cpp += &format!(
                "auto epok_split_{index}=({});for(unsigned i=0;i<{length};++i)epok_split_value.{}[i]=epok_split_{index}[i];",
                value.cpp, field.name
            );
        } else {
            cpp += &format!("epok_split_value.{}=({});", field.name, value.cpp);
        }
    }
    cpp += "return epok_split_value;}())";
    Ok(cpp)
}

fn project_fields(mut value: Expression, path: &str) -> Result<Expression, String> {
    if path.is_empty() {
        return Ok(value);
    }
    if path.split('.').count() > 16 {
        return Err("Split pin nesting exceeds 16 levels".into());
    }
    for name in path.split('.') {
        let fields = value.value_type.members();
        let (index, field) = fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name == name)
            .ok_or_else(|| format!("Unknown split output field {name}"))?;
        if !crate::scripts::identifier(name) {
            return Err("Invalid split field name".into());
        }
        value.cpp = if matches!(value.value_type, Type::Vector { .. }) {
            format!("({})[{index}]", value.cpp)
        } else {
            format!("({}).{name}", value.cpp)
        };
        value.value_type = field.value_type.clone();
        if let Type::Vector { length } = field.value_type {
            value.cpp = format!(
                "([&](){{const auto& epok_split_source=({});epok::bp::Vector<{length}> epok_split_result{{}};for(unsigned i=0;i<{length};++i)epok_split_result[i]=epok_split_source[i];return epok_split_result;}}())",
                value.cpp
            );
            value.lvalue = false;
        }
    }
    Ok(value)
}
struct Lower<'a, 'b> {
    signature: &'a schema::Function,
    context: &'a Context<'b>,
    nodes: BTreeMap<String, &'a Node>,
    values: BTreeSet<String>,
    exec: BTreeSet<String>,
    // Explicit execution wiring is an authored evaluation point even when a
    // callable's body is later proven read-only. Its output stays a snapshot.
    execution_nodes: BTreeSet<String>,
    budget: usize,
    available: BTreeSet<String>,
    temporaries: BTreeMap<String, Type>,
    data_budget: usize,
}
/// One effect policy for pure-body validation and conservative authored-helper
/// inference. Unknown operations/callees never establish a purity proof.
pub fn pure_node_read_only(
    node: &Node,
    registry: &crate::blueprint::Registry,
    functions: &BTreeMap<String, schema::Function>,
    parent_function: Option<&schema::Function>,
) -> bool {
    match &node.kind {
        NodeKind::SetVariable { .. }
        | NodeKind::Delay
        | NodeKind::WaitPlayback { .. }
        | NodeKind::Timeline { .. }
        | NodeKind::StopTimeline { .. } => false,
        NodeKind::Call { function } => functions.get(function).is_some_and(|f| f.pure),
        NodeKind::CallParent => parent_function.is_some_and(|f| f.pure),
        NodeKind::CallOn { class, function } => {
            call_on_function(registry, class, function).is_ok_and(|f| f.pure)
        }
        NodeKind::Builtin { operation } => builtin_signature(operation).2,
        NodeKind::Operation { operation } => registry.operation(operation).is_some_and(|op| {
            matches!(
                op.effect,
                schema::OperationEffect::PureValue | schema::OperationEffect::StateRead
            )
        }),
        _ => true,
    }
}
impl Lower<'_, '_> {
    fn validate_pure_node(&self, node: &Node) -> Result<(), Error> {
        if !self.signature.pure {
            return Ok(());
        }
        let read_only = pure_node_read_only(
            node,
            self.context.registry,
            self.context.functions,
            self.context.parent_function,
        );
        if read_only {
            Ok(())
        } else {
            Err(err(
                &node.id,
                "Pure functions cannot mutate state, consume events or suspend",
            ))
        }
    }
    fn operation_definition(&self, id: &str) -> Result<schema::Operation, Error> {
        self.context
            .registry
            .operation(id)
            .cloned()
            .ok_or_else(|| err(id, format!("Unknown gameplay operation {id}")))
    }
    fn operation(
        &mut self,
        node: &Node,
        operation: &schema::Operation,
    ) -> Result<Expression, Error> {
        let mut evaluation = String::new();
        let mut cpp = Vec::with_capacity(operation.parameters.len() + 1);
        let receiver: crate::operation::Receiver<Expression> = match &operation.receiver {
            schema::ReceiverKind::Service { library } => crate::operation::Receiver::Service {
                library: library.clone(),
            },
            schema::ReceiverKind::Instance { class } => {
                let value = self.input(node, "__target")?;
                let expected = Type::ObjectRef {
                    class: Some(class.clone()),
                };
                if !assignable(&value.value_type, &expected, self.context.registry) {
                    return Err(err(
                        &node.id,
                        "Receiver requires a compatible typed ObjectRef; use checked Cast first",
                    ));
                }
                evaluation.push_str(&format!("auto epok_receiver=({});", value.cpp));
                cpp.push("epok_receiver".into());
                crate::operation::Receiver::Instance {
                    value: value.clone(),
                    value_type: value.value_type.clone(),
                }
            }
            schema::ReceiverKind::Value { cpp_name } => {
                let value = self.input(node, "__target")?;
                let expected = Type::Record {
                    cpp_name: cpp_name.clone(),
                    fields: vec![],
                };
                if value.value_type.label() != expected.label() {
                    return Err(err(&node.id, "Value receiver has an incompatible type"));
                }
                evaluation.push_str(&format!("const auto& epok_receiver=({});", value.cpp));
                cpp.push("epok_receiver".into());
                crate::operation::Receiver::Value {
                    value: value.clone(),
                    value_type: value.value_type.clone(),
                }
            }
        };
        if operation.outputs.len() > 1 {
            return Err(err(
                &node.id,
                "Multiple operation outputs require a result record",
            ));
        }
        let mut lowered = Vec::with_capacity(operation.parameters.len());
        for (index, parameter) in operation.parameters.iter().enumerate() {
            let value = self.input(node, &parameter.id)?;
            if !assignable(
                &value.value_type,
                &parameter.value_type,
                self.context.registry,
            ) {
                return Err(err(
                    &node.id,
                    format!(
                        "Operation input {} requires {}",
                        parameter.name,
                        parameter.value_type.label()
                    ),
                ));
            }
            if parameter.direction == schema::Direction::MutableReference && !value.lvalue {
                return Err(err(
                    &node.id,
                    format!(
                        "Operation input {} requires mutable storage",
                        parameter.name
                    ),
                ));
            }
            let binding = match parameter.direction {
                schema::Direction::Value => "auto",
                schema::Direction::ConstReference => "const auto&",
                schema::Direction::MutableReference => "auto&",
            };
            evaluation.push_str(&format!("{binding} epok_argument_{index}=({});", value.cpp));
            cpp.push(format!("epok_argument_{index}"));
            lowered.push(crate::operation::Argument {
                parameter_id: parameter.id.clone(),
                value,
                value_type: parameter.value_type.clone(),
            });
        }
        let resolved = crate::operation::ResolvedCall::from_operation(
            operation,
            receiver,
            lowered,
            crate::script_ir::Span::default(),
        )
        .map_err(|message| err(&node.id, message))?;
        let target = match &operation.receiver {
            schema::ReceiverKind::Service { .. } => operation.native_target.clone(),
            schema::ReceiverKind::Instance { class } => call_on_name(class, &operation.id),
            schema::ReceiverKind::Value { .. } => operation.native_target.clone(),
        };
        let returns = operation
            .outputs
            .first()
            .map(|output| output.value_type.clone())
            .unwrap_or(Type::Void);
        let cpp_type = cpp_type(&returns).map_err(|message| err(&node.id, message))?;
        let _ = resolved;
        Ok(Expression {
            value_type: returns,
            cpp: format!(
                "([&]() -> {cpp_type} {{{evaluation}return {target}({});}}())",
                cpp.join(",")
            ),
            lvalue: false,
        })
    }
    fn checked_literal(
        &self,
        node: &str,
        value: &serde_json::Value,
        ty: &Type,
    ) -> Result<String, Error> {
        if let Type::ClassRef { base } = ty
            && (!self.context.registry.classes.contains_key(base)
                || value.as_str().is_some_and(|id| {
                    !crate::blueprint_refs::class_is_a(self.context.registry, id, base)
                }))
        {
            return Err(err(
                node,
                "Class reference is missing or is incompatible with its declared base",
            ));
        }
        literal(value, ty).map_err(|e| err(node, e))
    }
    fn builtin(&mut self, node: &Node, operation: &Builtin) -> Result<Expression, Error> {
        let (parameters, returns, _) = builtin_signature(operation);
        let mut args = vec![];
        for (name, ty) in parameters {
            let value = self.input(node, &name)?;
            if !assignable(&value.value_type, &ty, self.context.registry) {
                return Err(err(
                    &node.id,
                    format!("Builtin input {name} requires {}", ty.label()),
                ));
            }
            args.push(value.cpp);
        }
        if let Some(slots) = crate::blueprint_playback::slots(
            operation,
            self.context.playback_timelines,
            self.context.playback_effects,
        )
        .map_err(|e| err(&node.id, e))?
        {
            for slot in crate::blueprint_playback::external(slots) {
                let name = crate::blueprint_playback::pin(slot);
                if !node.inputs.contains_key(&name) {
                    if slot.required {
                        return Err(err(
                            &node.id,
                            format!(
                                "Required playback binding {} ({}) is missing",
                                slot.name, slot.id
                            ),
                        ));
                    }
                    args.push("epok::ObjectId{}".into());
                    continue;
                }
                if slot.required
                    && matches!(node.inputs.get(&name),Some(Input::Literal {value,..}) if value.is_null())
                {
                    return Err(err(
                        &node.id,
                        format!(
                            "Required playback binding {} ({}) cannot be null",
                            slot.name, slot.id
                        ),
                    ));
                }
                let value = self.input(node, &name)?;
                if !assignable(&value.value_type, &slot.target, self.context.registry) {
                    return Err(err(
                        &node.id,
                        format!(
                            "Playback binding {} ({}) requires {}",
                            slot.name,
                            slot.id,
                            slot.target.label()
                        ),
                    ));
                }
                args.push(value.cpp);
            }
            return Ok(Expression {
                value_type: returns,
                cpp: format!(
                    "{}({})",
                    crate::blueprint_playback::call_name(operation)
                        .ok_or_else(|| err(&node.id, "Playback asset requires a UUID"))?,
                    args.join(",")
                ),
                lvalue: false,
            });
        }
        let returns = if matches!(operation, Builtin::SelfObject) {
            let id = self
                .context
                .registry
                .named(self.context.self_class)
                .map(|c| c.id.clone());
            if self.context.self_kind == SelfKind::Actor {
                Type::ActorRef { class: id }
            } else {
                Type::ComponentRef { class: id }
            }
        } else if let Builtin::Cast { class } = operation {
            match self
                .context
                .model
                .and_then(|m| m.class(class))
                .map(|c| c.family)
            {
                Some(schema::ClassFamily::Actor) => Type::ActorRef {
                    class: Some(class.clone()),
                },
                Some(schema::ClassFamily::Component) => Type::ComponentRef {
                    class: Some(class.clone()),
                },
                _ => Type::ObjectRef {
                    class: Some(class.clone()),
                },
            }
        } else if matches!(operation, Builtin::GetOwner) {
            let owner = self
                .context
                .model
                .and_then(|m| m.class(self.context.self_class))
                .and_then(|c| c.component.as_ref());
            let class = owner
                .filter(|c| c.owners.len() == 1)
                .and_then(|c| c.owners.first())
                .and_then(|d| match d {
                    schema::Domain::World3D => Some(crate::object_model::ACTOR3D_ID.into()),
                    schema::Domain::World2D => Some(crate::object_model::ACTOR2D_ID.into()),
                    schema::Domain::UI => Some(crate::object_model::UI_ACTOR_ID.into()),
                    _ => None,
                });
            Type::ActorRef { class }
        } else {
            returns
        };
        let cpp =
            match operation {
                Builtin::SelfObject => builtin_cpp(operation, &args, &SelfReceiver::this("this"))
                    .map_err(|m| err(&node.id, m))?,
                Builtin::GetOwner => {
                    if self.context.self_kind != SelfKind::Component {
                        return Err(err(
                            &node.id,
                            "Get Owner is only available inside a Component Blueprint",
                        ));
                    }
                    builtin_cpp(operation, &args, &SelfReceiver::this("this"))
                        .map_err(|m| err(&node.id, m))?
                }
                Builtin::SpawnActor { base } => {
                    let metadata = self
                        .context
                        .registry
                        .classes
                        .get(base)
                        .ok_or_else(|| err(&node.id, "Unknown spawn base class"))?;
                    let family = self
                        .context
                        .model
                        .and_then(|model| model.class(base))
                        .map(|class| class.family)
                        .unwrap_or_default();
                    if family != schema::ClassFamily::Actor {
                        return Err(err(
                            &node.id,
                            format!(
                                "Spawn Actor requires an Actor class; {} belongs to the {} family",
                                metadata.cpp_name,
                                family.label()
                            ),
                        ));
                    }
                    builtin_cpp(
                        operation,
                        &args,
                        &SelfReceiver::this(self.context.self_kind.actor()),
                    )
                    .map_err(|m| err(&node.id, m))?
                }
                Builtin::Spawn { class } | Builtin::IsA { class } | Builtin::Cast { class } => {
                    let metadata =
                        self.context.registry.classes.get(class).ok_or_else(|| {
                            err(&node.id, format!("Unknown class reference {class}"))
                        })?;
                    if matches!(operation, Builtin::Spawn { .. }) && metadata.abstract_class {
                        return Err(err(&node.id, "Cannot spawn an abstract class"));
                    }
                    let id = crate::blueprint_refs::compact_id(class);
                    let _ = id;
                    if matches!(operation, Builtin::Spawn { .. })
                        && self
                            .context
                            .model
                            .and_then(|m| m.class(class))
                            .is_none_or(|c| {
                                c.family != schema::ClassFamily::Actor || !c.placement.spawnable
                            })
                    {
                        return Err(err(&node.id, "Spawn requires a spawnable Actor class"));
                    }
                    builtin_cpp(
                        operation,
                        &args,
                        &SelfReceiver::this(self.context.self_kind.actor()),
                    )
                    .map_err(|m| err(&node.id, m))?
                }
                Builtin::SpawnClass { base } => {
                    let metadata = self
                        .context
                        .registry
                        .classes
                        .get(base)
                        .ok_or_else(|| err(&node.id, "Unknown spawn base class"))?;
                    if metadata.backend != schema::native_backend()
                        || metadata.provider.version != 1
                        || !matches!(metadata.provider.id.as_str(), "cpp" | "blueprint")
                    {
                        return Err(err(
                            &node.id,
                            "Spawn base requires a supported native class representation",
                        ));
                    }
                    builtin_cpp(
                        operation,
                        &args,
                        &SelfReceiver::this(self.context.self_kind.actor()),
                    )
                    .map_err(|m| err(&node.id, m))?
                }
                _ => builtin_cpp(
                    operation,
                    &args,
                    &SelfReceiver::this(self.context.self_kind.actor()),
                )
                .map_err(|m| err(&node.id, m))?,
            };
        Ok(Expression {
            value_type: returns,
            cpp,
            lvalue: false,
        })
    }
    fn input(&mut self, node: &Node, name: &str) -> Result<Expression, Error> {
        if name.split('.').count() > 16 {
            return Err(err(&node.id, "Maximum split depth exceeded"));
        }
        // Deleted source nodes and pasted external wires leave child defaults.
        if !node.inputs.contains_key(name)
            && let Some((root, path)) = name.split_once('.')
            && let Some(Input::Literal { value_type, .. }) = node.inputs.get(root)
            && let Some(ty) = value_type.member_type(path)
        {
            let value = crate::script_values::default_value(&ty);
            return Ok(Expression {
                cpp: self.checked_literal(&node.id, &value, &ty)?,
                value_type: ty,
                lvalue: false,
            });
        }
        let input = node
            .inputs
            .get(name)
            .ok_or_else(|| {
                let hint = if name == "target" || name == "__target" {
                    ". Connect an Actor to Target; use Self for an Actor Blueprint or Get Owner for an ActorComponent Blueprint."
                } else { "" };
                err(&node.id, format!("Missing input {name}{hint}"))
            })?;
        if let Input::Literal { value_type, .. } = input {
            let prefix = format!("{name}.");
            if node.inputs.keys().any(|key| key.starts_with(&prefix)) {
                let values = value_type
                    .members()
                    .iter()
                    .map(|field| self.input(node, &format!("{name}.{}", field.name)))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Expression {
                    value_type: value_type.clone(),
                    cpp: compose(value_type, &values).map_err(|e| err(&node.id, e))?,
                    lvalue: false,
                });
            }
        }
        match input {
            Input::Literal { value_type, value } => Ok(Expression {
                value_type: value_type.clone(),
                cpp: self.checked_literal(&node.id, value, value_type)?,
                lvalue: false,
            }),
            Input::Parameter { name } => {
                let p = self
                    .signature
                    .parameters
                    .iter()
                    .find(|p| p.name == *name)
                    .ok_or_else(|| err(&node.id, format!("Unknown parameter {name}")))?;
                Ok(Expression {
                    value_type: p.value_type.clone(),
                    cpp: p.name.clone(),
                    lvalue: p.direction != schema::Direction::ConstReference,
                })
            }
            Input::Link { node: source, pin } => {
                let (root_pin, fields) = pin.split_once('.').unwrap_or((pin, ""));
                if self
                    .nodes
                    .get(source)
                    .is_some_and(|n| matches!(n.kind, NodeKind::Entry))
                {
                    let p = self
                        .signature
                        .parameters
                        .iter()
                        .find(|p| p.name == root_pin)
                        .ok_or_else(|| err(&node.id, "Unknown Entry parameter pin"))?;
                    return project_fields(
                        Expression {
                            value_type: p.value_type.clone(),
                            cpp: p.name.clone(),
                            lvalue: p.direction != schema::Direction::ConstReference,
                        },
                        fields,
                    )
                    .map_err(|e| err(&node.id, e));
                }
                if root_pin != "value" {
                    return Err(err(&node.id, format!("Unknown value output pin {pin}")));
                }
                let value = self.expression(source)?;
                project_fields(value, fields).map_err(|e| err(&node.id, e))
            }
        }
    }
    fn property(&self, node: &Node, member: &str) -> Result<schema::Property, Error> {
        self.context.properties.get(member).cloned().ok_or_else(|| {
            err(
                &node.id,
                format!("Unknown property ID {member}; broken reference preserved"),
            )
        })
    }
    fn expression(&mut self, id: &str) -> Result<Expression, Error> {
        if self.data_budget == 0 || self.values.len() >= 128 {
            return Err(err(
                id,
                "Expanded data expressions exceed 4096 nodes or 128 levels; use explicit variables to share computation",
            ));
        }
        self.data_budget -= 1;
        if !self.values.insert(id.into()) {
            return Err(err(id, "Data cycle is not permitted"));
        }
        let n = *self
            .nodes
            .get(id)
            .ok_or_else(|| err(id, "Missing source node"))?;
        self.validate_pure_node(n)?;
        let result = match &n.kind {
            NodeKind::Literal { value_type, value } => Ok(Expression {
                value_type: value_type.clone(),
                cpp: self.checked_literal(id, value, value_type)?,
                lvalue: false,
            }),
            NodeKind::GetVariable { member } => {
                let p = self.property(n, member)?;
                let cpp = if let Type::Vector { length } = p.value_type {
                    format!(
                        "epok::bp::Vector<{length}>{{{{{}}}}}",
                        (0..length)
                            .map(|i| format!("this->{}[{i}]", p.name))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                } else {
                    format!("this->{}", p.name)
                };
                Ok(Expression {
                    value_type: p.value_type,
                    cpp,
                    lvalue: p.editable || self.context.writable.contains(&p.id),
                })
            }
            NodeKind::Reroute => self.input(n, "value"),
            NodeKind::Not => {
                let v = self.input(n, "value")?;
                if v.value_type != Type::Bool {
                    return Err(err(id, "Not requires Bool"));
                }
                Ok(Expression {
                    value_type: Type::Bool,
                    cpp: format!("(!({}))", v.cpp),
                    lvalue: false,
                })
            }
            NodeKind::Binary { op } => {
                let a = self.input(n, "a")?;
                let b = self.input(n, "b")?;
                binary(*op, a, b).map_err(|e| err(id, e))
            }
            NodeKind::MakeVector { length } => {
                if !matches!(length, 2 | 3) {
                    return Err(err(id, "Make Vector supports exactly 2 or 3 components"));
                }
                let mut values = vec![];
                for port in ["x", "y", "z"].iter().take(*length) {
                    let value = self.input(n, port)?;
                    if value.value_type != Type::Fixed {
                        return Err(err(id, format!("Vector component {port} requires Fixed")));
                    }
                    values.push(value.cpp);
                }
                Ok(Expression {
                    value_type: Type::Vector { length: *length },
                    cpp: format!("epok::bp::Vector<{length}>{{{{{}}}}}", values.join(",")),
                    lvalue: false,
                })
            }
            NodeKind::VectorComponent { length, index } => {
                if !matches!(length, 2 | 3) || index >= length {
                    return Err(err(
                        id,
                        "Vector component index must be inside a 2D or 3D vector",
                    ));
                }
                let value = self.input(n, "value")?;
                if value.value_type != (Type::Vector { length: *length }) {
                    return Err(err(
                        id,
                        "Vector component input has the wrong vector length",
                    ));
                }
                Ok(Expression {
                    value_type: Type::Fixed,
                    cpp: format!("({})[{index}]", value.cpp),
                    lvalue: false,
                })
            }
            NodeKind::Call { function } => {
                let f = self
                    .context
                    .functions
                    .get(function)
                    .ok_or_else(|| err(id, format!("Unknown function ID {function}")))?;
                if !f.callable || f.access != "public" {
                    return Err(err(
                        id,
                        "Data calls require a public BlueprintPure function; impure calls need execution links",
                    ));
                }
                if self.available.contains(id) && f.returns != Type::Void {
                    self.values.remove(id);
                    return Ok(Expression {
                        value_type: f.returns.clone(),
                        cpp: temporary(id),
                        lvalue: true,
                    });
                }
                if !f.pure || self.execution_nodes.contains(id) {
                    return Err(err(
                        id,
                        "Impure call result is only available after its execution on every incoming path",
                    ));
                }
                let f = f.clone();
                self.call(n, &f, false)
            }
            NodeKind::CallParent => {
                let f = self
                    .context
                    .parent_function
                    .ok_or_else(|| err(id, "Call Parent is only valid in an override"))?;
                if !self.available.contains(id) || f.returns == Type::Void {
                    return Err(err(id, "Parent result requires prior execution"));
                }
                Ok(Expression {
                    value_type: f.returns.clone(),
                    cpp: temporary(id),
                    lvalue: true,
                })
            }
            NodeKind::Builtin { operation } => {
                let (_, returns, pure) = builtin_signature(operation);
                if pure {
                    self.builtin(n, operation)
                } else if returns != Type::Void && self.available.contains(id) {
                    Ok(Expression {
                        value_type: returns,
                        cpp: temporary(id),
                        lvalue: true,
                    })
                } else {
                    Err(err(
                        id,
                        "Impure builtin result requires prior execution on every incoming path",
                    ))
                }
            }
            NodeKind::Operation { operation } => {
                let definition = self.operation_definition(operation)?;
                let returns = definition
                    .outputs
                    .first()
                    .map(|output| output.value_type.clone())
                    .unwrap_or(Type::Void);
                let pure = matches!(
                    definition.effect,
                    schema::OperationEffect::PureValue | schema::OperationEffect::StateRead
                );
                if pure {
                    self.operation(n, &definition)
                } else if returns != Type::Void && self.available.contains(id) {
                    Ok(Expression {
                        value_type: returns,
                        cpp: temporary(id),
                        lvalue: true,
                    })
                } else {
                    Err(err(
                        id,
                        "Impure operation result requires prior execution on every incoming path",
                    ))
                }
            }
            NodeKind::CallOn { class, function } => {
                let f = call_on_function(self.context.registry, class, function)
                    .map_err(|e| err(id, e))?;
                if self.available.contains(id) && f.returns != Type::Void {
                    Ok(Expression {
                        value_type: f.returns,
                        cpp: temporary(id),
                        lvalue: true,
                    })
                } else if !f.pure || self.execution_nodes.contains(id) {
                    Err(err(
                        id,
                        "Impure receiver call result requires prior execution on every incoming path",
                    ))
                } else {
                    self.call_on(n, class, function, &f)
                }
            }
            _ => Err(err(id, "Node has no pure value output")),
        };
        self.values.remove(id);
        result
    }
    fn call(
        &mut self,
        node: &Node,
        f: &schema::Function,
        parent: bool,
    ) -> Result<Expression, Error> {
        let callee = format!(
            "{}{}",
            if parent {
                format!("{}::", self.context.parent_name)
            } else {
                "this->".into()
            },
            f.name
        );
        self.invoke(node, f, &callee, None)
    }
    fn call_on(
        &mut self,
        node: &Node,
        class: &str,
        function: &str,
        f: &schema::Function,
    ) -> Result<Expression, Error> {
        let receiver = self.input(node, "__target")?;
        if !assignable(
            &receiver.value_type,
            &Type::ObjectRef {
                class: Some(class.into()),
            },
            self.context.registry,
        ) {
            return Err(err(
                &node.id,
                "Receiver requires a compatible typed ObjectRef; use checked Cast first",
            ));
        }
        self.invoke(node, f, &call_on_name(class, function), Some(receiver))
    }
    fn invoke(
        &mut self,
        node: &Node,
        f: &schema::Function,
        callee: &str,
        receiver: Option<Expression>,
    ) -> Result<Expression, Error> {
        let mut args = vec![];
        let mut evaluation = String::new();
        if let Some(receiver) = receiver {
            evaluation.push_str(&format!("auto epok_receiver=({});", receiver.cpp));
            args.push("epok_receiver".into());
        }
        for (index, p) in f.parameters.iter().enumerate() {
            let v = self.input(node, &p.name)?;
            if !assignable(&v.value_type, &p.value_type, self.context.registry) {
                return Err(err(
                    &node.id,
                    format!(
                        "Argument {} expects {}, got {}",
                        p.name,
                        p.value_type.label(),
                        v.value_type.label()
                    ),
                ));
            }
            if p.direction == schema::Direction::MutableReference && !v.lvalue {
                return Err(err(
                    &node.id,
                    format!("Argument {} requires mutable storage", p.name),
                ));
            }
            let binding = match p.direction {
                schema::Direction::Value => "auto",
                schema::Direction::ConstReference => "const auto&",
                schema::Direction::MutableReference => "auto&",
            };
            evaluation.push_str(&format!("{binding} epok_argument_{index}=({});", v.cpp));
            args.push(format!("epok_argument_{index}"));
        }
        Ok(Expression {
            value_type: f.returns.clone(),
            cpp: format!(
                "([&]() -> {} {{{evaluation}return {callee}({});}}())",
                cpp_type(&f.returns).map_err(|e| err(&node.id, e))?,
                args.join(",")
            ),
            lvalue: false,
        })
    }
    fn edges(&mut self, node: &Node, port: &str) -> Result<Vec<Statement>, Error> {
        let mut out = vec![];
        if let Some(ids) = node.outputs.get(port) {
            if ids.len() > 1 {
                return Err(err(
                    &node.id,
                    format!(
                        "Execution output {port} has more than one connection; use separate Sequence outputs."
                    ),
                ));
            }
            for id in ids {
                if self
                    .nodes
                    .get(id)
                    .is_some_and(|target| matches!(target.kind, NodeKind::Entry))
                {
                    return Err(err(
                        &node.id,
                        "Event and function Entry nodes do not accept incoming execution connections",
                    ));
                }
                out.extend(self.block(id)?);
            }
        }
        Ok(out)
    }
    fn block(&mut self, id: &str) -> Result<Vec<Statement>, Error> {
        if self.exec.len() >= 128 {
            return Err(err(id, "Execution nesting exceeds 128 levels"));
        }
        if self.budget == 0 {
            return Err(err(id, "Expanded graph exceeds 4096 execution nodes"));
        }
        self.budget -= 1;
        if !self.exec.insert(id.into()) {
            return Err(err(id, "Execution cycles must use the bounded Loop node"));
        }
        let node = *self
            .nodes
            .get(id)
            .ok_or_else(|| err(id, "Execution link targets a missing node"))?;
        self.validate_pure_node(node)?;
        let mut out = vec![Statement::Node(id.into())];
        match &node.kind {
            NodeKind::Entry => {}
            NodeKind::Sequence => {
                for pin in crate::blueprint_asset::sequence_outputs(node) {
                    out.extend(self.edges(node, &pin)?);
                }
            }
            NodeKind::Reroute if node.inputs.is_empty() => {
                out.clear();
            }
            NodeKind::SetVariable { member } => {
                let p = self.property(node, member)?;
                if !p.editable && !self.context.writable.contains(&p.id) {
                    return Err(err(id, "Read-only property cannot be assigned"));
                }
                let value = self.input(node, "value")?;
                if !assignable(&value.value_type, &p.value_type, self.context.registry) {
                    return Err(err(
                        id,
                        format!(
                            "Assignment expects {}, got {}",
                            p.value_type.label(),
                            value.value_type.label()
                        ),
                    ));
                }
                out.push(Statement::Assign {
                    target: format!("this->{}", p.name),
                    value,
                });
            }
            NodeKind::Call { function } | NodeKind::CallOn { function, .. } => {
                let f = if let NodeKind::CallOn { class, .. } = &node.kind {
                    call_on_function(self.context.registry, class, function)
                        .map_err(|e| err(id, e))?
                } else {
                    self.context
                        .functions
                        .get(function)
                        .ok_or_else(|| err(id, format!("Unknown function ID {function}")))?
                        .clone()
                };
                if !f.callable || f.access != "public" {
                    return Err(err(id, "Function is not public BlueprintCallable"));
                }
                let call = if let NodeKind::CallOn { class, .. } = &node.kind {
                    self.call_on(node, class, function, &f)?
                } else {
                    self.call(node, &f, false)?
                };
                if f.returns != Type::Void {
                    self.temporaries.insert(temporary(id), f.returns.clone());
                    self.available.insert(id.into());
                    out.push(Statement::Evaluate(format!(
                        "{}={}",
                        temporary(id),
                        call.cpp
                    )));
                } else {
                    out.push(Statement::Evaluate(call.cpp));
                }
                out.push(Statement::CheckOwner(self.signature.returns.clone()));
            }
            NodeKind::CallParent => {
                let f = self
                    .context
                    .parent_function
                    .ok_or_else(|| {
                        err(id, "Call Parent is only available inside an event override")
                    })?
                    .clone();
                if f.abstract_method {
                    return Err(err(
                        id,
                        "The parent event is abstract and has no implementation",
                    ));
                }
                let call = self.call(node, &f, true)?;
                if f.returns != Type::Void {
                    self.temporaries.insert(temporary(id), f.returns.clone());
                    self.available.insert(id.into());
                    out.push(Statement::Evaluate(format!(
                        "{}={}",
                        temporary(id),
                        call.cpp
                    )));
                } else {
                    out.push(Statement::Evaluate(call.cpp));
                }
                out.push(Statement::CheckOwner(self.signature.returns.clone()));
            }
            NodeKind::Branch => {
                let condition = self.input(node, "condition")?;
                if condition.value_type != Type::Bool {
                    return Err(err(id, "Branch condition must be Bool"));
                }
                let before = self.available.clone();
                let yes = self.edges(node, "true")?;
                let yes_available = self.available.clone();
                self.available = before;
                let no = self.edges(node, "false")?;
                self.available = self
                    .available
                    .intersection(&yes_available)
                    .cloned()
                    .collect();
                out.push(Statement::Branch { condition, yes, no });
            }
            NodeKind::Loop { count } => {
                if *count > 1024 {
                    return Err(err(id, "Loop count must be at most 1024"));
                }
                let before = self.available.clone();
                let body = self.edges(node, "body")?;
                if *count == 0 {
                    self.available = before;
                }
                out.push(Statement::Loop {
                    count: *count,
                    body,
                });
            }
            NodeKind::Delay => {
                if self.signature.returns != Type::Void {
                    return Err(err(
                        id,
                        "Delay requires a void event; synchronous value-returning calls cannot suspend",
                    ));
                }
                let seconds = self.input(node, "seconds")?;
                if seconds.value_type != Type::Fixed {
                    return Err(err(id, "Delay duration requires Fixed seconds"));
                }
                out.push(Statement::Delay(seconds));
            }
            NodeKind::WaitPlayback { condition } => {
                if self.signature.returns != Type::Void {
                    return Err(err(
                        id,
                        "Playback waits require a void event; synchronous calls cannot suspend",
                    ));
                }
                let target = self.input(node, "playback")?;
                let required = if matches!(
                    condition,
                    crate::blueprint_asset::PlaybackCondition::EffectComplete
                ) {
                    Type::EffectHandle
                } else {
                    Type::SequenceHandle
                };
                if target.value_type != required {
                    return Err(err(
                        id,
                        format!("Playback wait requires {}", required.label()),
                    ));
                }
                let (asset, marker) = match condition {
                    crate::blueprint_asset::PlaybackCondition::Marker { timeline, marker }
                    | crate::blueprint_asset::PlaybackCondition::SubscribeMarker {
                        timeline,
                        marker,
                    } => {
                        for value in [timeline, marker] {
                            if !uuid::Uuid::parse_str(value)
                                .is_ok_and(|id| !id.is_nil() && id.to_string() == *value)
                            {
                                return Err(err(
                                    id,
                                    "Marker wait requires canonical persistent timeline and marker UUIDs",
                                ));
                            }
                        }
                        (
                            crate::blueprint_refs::compact_id(timeline),
                            crate::blueprint_refs::compact_id(marker),
                        )
                    }
                    _ => (0, 0),
                };
                let before = self.available.clone();
                let reached = self.edges(node, "reached")?;
                self.available = before.clone();
                let completed = self.edges(node, "completed")?;
                self.available = before.clone();
                let cancelled = self.edges(node, "cancelled")?;
                self.available = before;
                out.push(Statement::WaitPlayback {
                    node: node.id.clone(),
                    repeating: matches!(
                        condition,
                        crate::blueprint_asset::PlaybackCondition::SubscribeMarker { .. }
                    ),
                    target,
                    asset,
                    marker,
                    reached,
                    completed,
                    cancelled,
                });
            }
            NodeKind::Timeline {
                keys,
                looping,
                member,
            } => {
                if self.signature.returns != Type::Void {
                    return Err(err(id, "Timeline requires a void event"));
                }
                if keys.len() < 2
                    || keys.len() > 16
                    || keys[0][0] != 0.
                    || keys
                        .windows(2)
                        .any(|pair| (pair[0][0] * 4096.).round() >= (pair[1][0] * 4096.).round())
                {
                    return Err(err(
                        id,
                        "Timeline requires 2–16 increasing Q12-representable times starting at zero",
                    ));
                }
                let p = self.property(node, member)?;
                if p.value_type != Type::Fixed
                    || (!p.editable && !self.context.writable.contains(member))
                {
                    return Err(err(id, "Timeline target must be a writable Fixed property"));
                }
                let keys = keys
                    .iter()
                    .map(|key| {
                        Ok((
                            literal(&serde_json::json!(key[0]), &Type::Fixed)
                                .map_err(|e| err(id, e))?,
                            literal(&serde_json::json!(key[1]), &Type::Fixed)
                                .map_err(|e| err(id, e))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                let available = self.available.clone();
                let updated = self.edges(node, "updated")?;
                self.available = available.clone();
                let finished = self.edges(node, "finished")?;
                self.available = available;
                out.push(Statement::Timeline {
                    node: id.into(),
                    target: format!("this->{}", p.name),
                    keys,
                    looping: *looping,
                    updated,
                    finished,
                });
            }
            NodeKind::StopTimeline { node: target } => {
                if !self
                    .nodes
                    .get(target)
                    .is_some_and(|n| matches!(n.kind, NodeKind::Timeline { .. }))
                {
                    return Err(err(
                        id,
                        "Stop Timeline must reference a Timeline node in this graph",
                    ));
                }
                out.push(Statement::StopTimeline(target.clone()));
            }
            NodeKind::Builtin { operation } => {
                let (_, returns, pure) = builtin_signature(operation);
                if pure {
                    return Err(err(id, "Pure builtin cannot consume execution links"));
                }
                let value = self.builtin(node, operation)?;
                if returns != Type::Void {
                    self.available.insert(id.into());
                    self.temporaries.insert(temporary(id), returns);
                    out.push(Statement::Evaluate(format!(
                        "{}={}",
                        temporary(id),
                        value.cpp
                    )));
                } else {
                    out.push(Statement::Evaluate(value.cpp));
                }
                out.push(Statement::CheckOwner(self.signature.returns.clone()));
            }
            NodeKind::Operation { operation } => {
                let definition = self.operation_definition(operation)?;
                if matches!(
                    definition.effect,
                    schema::OperationEffect::PureValue | schema::OperationEffect::StateRead
                ) {
                    return Err(err(id, "Pure operation cannot consume execution links"));
                }
                let value = self.operation(node, &definition)?;
                if value.value_type != Type::Void {
                    self.available.insert(id.into());
                    self.temporaries
                        .insert(temporary(id), value.value_type.clone());
                    out.push(Statement::Evaluate(format!(
                        "{}={}",
                        temporary(id),
                        value.cpp
                    )));
                } else {
                    out.push(Statement::Evaluate(value.cpp));
                }
                if definition.can_destroy_receiver {
                    out.push(Statement::CheckOwner(self.signature.returns.clone()));
                }
            }
            NodeKind::Return => {
                let value = if self.signature.returns == Type::Void {
                    None
                } else {
                    let v = self.input(node, "value")?;
                    if v.value_type != self.signature.returns {
                        return Err(err(id, "Return type mismatch"));
                    }
                    Some(v)
                };
                out.push(Statement::Return(value));
                if node.outputs.values().any(|v| !v.is_empty()) {
                    return Err(err(id, "Return cannot have outgoing execution links"));
                }
                self.exec.remove(id);
                return Ok(out);
            }
            _ => {
                return Err(err(
                    id,
                    "Pure data node cannot be connected to an execution pin",
                ));
            }
        }
        let sequence_pins = crate::blueprint_asset::sequence_outputs(node);
        let allowed = match node.kind {
            NodeKind::Sequence => sequence_pins.iter().map(String::as_str).collect(),
            NodeKind::Branch => vec!["true", "false"],
            NodeKind::Loop { .. } => vec!["next", "body"],
            NodeKind::Timeline { .. } => vec!["next", "updated", "finished"],
            NodeKind::WaitPlayback { .. } => vec!["next", "reached", "completed", "cancelled"],
            _ => vec!["next"],
        };
        if node.outputs.keys().any(|p| !allowed.contains(&p.as_str())) {
            return Err(err(id, "Unknown execution output pin"));
        }
        out.extend(self.edges(node, "next")?);
        self.exec.remove(id);
        Ok(out)
    }
}
pub fn binary(op: BinaryOp, a: Expression, b: Expression) -> Result<Expression, String> {
    // Two references are the same object or not, whatever class each pin names:
    // every reference is one `epok::ObjectId`.
    if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
        && a.value_type.is_reference()
        && b.value_type.is_reference()
    {
        let token = if matches!(op, BinaryOp::Equal) {
            "=="
        } else {
            "!="
        };
        return Ok(Expression {
            value_type: Type::Bool,
            cpp: format!("(({}){token}({}))", a.cpp, b.cpp),
            lvalue: false,
        });
    }
    if a.value_type != b.value_type {
        return Err("Binary inputs must have exactly the same type".into());
    }
    let (cpp, ty) = match op {
        BinaryOp::And | BinaryOp::Or if a.value_type == Type::Bool => (
            format!(
                "(({}){}({}))",
                a.cpp,
                if matches!(op, BinaryOp::And) {
                    "&&"
                } else {
                    "||"
                },
                b.cpp
            ),
            Type::Bool,
        ),
        BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual
            if matches!(
                a.value_type,
                Type::Bool | Type::Int32 | Type::UInt32 | Type::Fixed | Type::Enum { .. }
            ) =>
        {
            let token = match op {
                BinaryOp::Equal => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Greater => ">",
                _ => ">=",
            };
            (format!("(({}){token}({}))", a.cpp, b.cpp), Type::Bool)
        }
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
            if matches!(
                a.value_type,
                Type::Int32 | Type::UInt32 | Type::Fixed | Type::Vector { .. }
            ) =>
        {
            let prefix = match a.value_type {
                Type::Int32 => "i",
                Type::UInt32 => "u",
                _ => "",
            };
            let op = match op {
                BinaryOp::Add => "add",
                BinaryOp::Subtract => "sub",
                BinaryOp::Multiply => "mul",
                _ => "div",
            };
            (
                format!("epok::bp::{prefix}{op}({}, {})", a.cpp, b.cpp),
                a.value_type,
            )
        }
        _ => return Err("Unsupported operator for the declared type".into()),
    };
    Ok(Expression {
        value_type: ty,
        cpp,
        lvalue: false,
    })
}
fn returns(body: &[Statement]) -> bool {
    body.iter().any(|s| match s {
        Statement::Return(_) => true,
        Statement::Branch { yes, no, .. } => returns(yes) && returns(no),
        _ => false,
    })
}
/// Resolve a foreign receiver's public native method, including inherited IDs.
pub fn call_on_function(
    registry: &crate::blueprint::Registry,
    class: &str,
    function: &str,
) -> Result<schema::Function, String> {
    let class = registry
        .classes
        .get(class)
        .ok_or("Unknown receiver class")?;
    if class.backend != schema::native_backend()
        || class.provider.version != 1
        || !matches!(class.provider.id.as_str(), "cpp" | "blueprint")
    {
        return Err("Receiver has no supported physical native C++ representation".into());
    }
    let mut functions = BTreeMap::<String, schema::Function>::new();
    for ancestor in registry.ancestry(&class.cpp_name) {
        for original in &ancestor.functions {
            let mut function = original.clone();
            for id in &original.overrides {
                if let Some(inherited) = functions.get(id) {
                    function.callable |= inherited.callable;
                    function.timeline = function.timeline.or(inherited.timeline);
                    function.pure |= inherited.pure;
                }
            }
            for id in function
                .overrides
                .iter()
                .chain(std::iter::once(&function.id))
            {
                functions.insert(id.clone(), function.clone());
            }
        }
    }
    let function = functions
        .get(function)
        .ok_or("Unknown function on receiver class")?
        .clone();
    if !function.callable || function.access != "public" {
        return Err("Receiver function is not public BlueprintCallable".into());
    }
    if function.parameters.iter().any(|p| p.name == "__target") {
        return Err("Receiver function uses reserved __target argument name".into());
    }
    Ok(function)
}
pub fn call_on_name(class: &str, function: &str) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "epok_call_on_{:x}",
        Sha256::digest(format!("{class}\0{function}").as_bytes())
    )
}
pub fn lower(
    graph: &Graph,
    signature: &schema::Function,
    context: &Context<'_>,
) -> Result<FunctionIr, Error> {
    let mut active_graph = graph.clone();
    active_graph.nodes = graph.compilation_nodes().cloned().collect();
    let graph = &active_graph;
    if graph.nodes.len() > 1024 {
        return Err(Error {
            node: None,
            message: "Graph exceeds 1024 nodes".into(),
        });
    }
    let mut nodes = BTreeMap::new();
    for n in &graph.nodes {
        if uuid::Uuid::parse_str(&n.id).is_err() {
            return Err(err(&n.id, "Node ID must be a UUID"));
        }
        if nodes.insert(n.id.clone(), n).is_some() {
            return Err(err(&n.id, "Duplicate node ID"));
        }
        let allowed = match &n.kind {
            NodeKind::SetVariable { .. }
            | NodeKind::Not
            | NodeKind::Reroute
            | NodeKind::VectorComponent { .. } => {
                vec!["value".to_string()]
            }
            NodeKind::Binary { .. } => vec!["a".into(), "b".into()],
            NodeKind::MakeVector { length } => ["x", "y", "z"]
                .iter()
                .take(*length)
                .map(|name| name.to_string())
                .collect(),
            NodeKind::Branch => vec!["condition".into()],
            NodeKind::Delay => vec!["seconds".into()],
            NodeKind::WaitPlayback { .. } => vec!["playback".into()],
            NodeKind::Return if signature.returns != Type::Void => vec!["value".into()],
            NodeKind::Call { function } => context
                .functions
                .get(function)
                .map(|f| f.parameters.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default(),
            NodeKind::CallOn { class, function } => {
                let f = call_on_function(context.registry, class, function)
                    .map_err(|e| err(&n.id, e))?;
                std::iter::once("__target".into())
                    .chain(f.parameters.iter().map(|p| p.name.clone()))
                    .collect()
            }
            NodeKind::CallParent => context
                .parent_function
                .map(|f| f.parameters.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default(),
            NodeKind::Builtin { operation } => {
                let mut names = builtin_signature(operation)
                    .0
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect::<Vec<_>>();
                if let Some(slots) = crate::blueprint_playback::slots(
                    operation,
                    context.playback_timelines,
                    context.playback_effects,
                )
                .map_err(|e| err(&n.id, e))?
                {
                    names.extend(
                        crate::blueprint_playback::external(slots)
                            .into_iter()
                            .map(crate::blueprint_playback::pin),
                    );
                }
                names
            }
            NodeKind::Operation { operation } => context
                .registry
                .operation(operation)
                .map(|operation| {
                    let mut inputs = operation
                        .parameters
                        .iter()
                        .map(|parameter| parameter.id.clone())
                        .collect::<Vec<_>>();
                    if !matches!(operation.receiver, schema::ReceiverKind::Service { .. }) {
                        inputs.insert(0, "__target".into());
                    }
                    inputs
                })
                .unwrap_or_default(),
            _ => vec![],
        };
        if let Some(name) = n.inputs.keys().find(|name| !allowed.contains(name) && !name.split_once('.').is_some_and(|(root, path)| {
            allowed.iter().any(|name| name == root) && matches!(n.inputs.get(root), Some(Input::Literal { value_type, .. }) if value_type.member_type(path).is_some())
        })) {
            return Err(err(
                &n.id,
                format!("Unknown input pin {name}; stale connection preserved"),
            ));
        }
    }
    if graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Entry))
        .count()
        != 1
    {
        return Err(err(
            &graph.entry,
            "Each graph requires exactly one Entry node",
        ));
    }
    if !nodes
        .get(&graph.entry)
        .is_some_and(|n| matches!(n.kind, NodeKind::Entry))
    {
        return Err(err(&graph.entry, "Graph entry must identify an Entry node"));
    }
    let mut execution_nodes = BTreeSet::new();
    let mut pending = vec![graph.entry.clone()];
    while let Some(id) = pending.pop() {
        if execution_nodes.insert(id.clone())
            && let Some(node) = nodes.get(&id)
        {
            pending.extend(node.outputs.values().flatten().cloned());
        }
    }
    let mut lower = Lower {
        signature,
        context,
        nodes,
        values: BTreeSet::new(),
        exec: BTreeSet::new(),
        execution_nodes,
        budget: 4096,
        available: BTreeSet::new(),
        temporaries: BTreeMap::new(),
        data_budget: 4096,
    };
    let body = lower.block(&graph.entry)?;
    if signature.returns != Type::Void && !returns(&body) {
        return Err(Error {
            node: None,
            message: "Every execution path must return a value".into(),
        });
    }
    let body = factor_shared_branch_tails(body);
    let body = if signature.pure {
        hoist_pure_calls(body)
    } else {
        body
    };
    Ok(FunctionIr {
        signature: signature.clone(),
        body,
        temporaries: lower.temporaries,
    })
}
fn temporary(node: &str) -> String {
    format!("epok_result_{}", node.replace('-', ""))
}

/// A wired pure node is spelled at every consumer, so a graph that reads one
/// result in several places runs the call that many times. Inside a pure
/// function nothing can change between those reads, so each call needs one
/// evaluation. A local is placed only where the call is very busy: every path
/// onward evaluates it and one path evaluates it twice. Calls a branch or a
/// short circuit guarded therefore never run earlier than they did.
pub(crate) fn hoist_pure_calls(body: Vec<Statement>) -> Vec<Statement> {
    const CALL: &str = "([&]() -> ";
    // Whole-function scans are quadratic; a graph this large keeps its text.
    const MAXIMUM_BODY_BYTES: usize = 256 * 1024;

    fn hoistable(body: &[Statement]) -> bool {
        body.iter().all(|statement| match statement {
            Statement::Node(_) | Statement::Return(_) => true,
            Statement::Branch { yes, no, .. } => hoistable(yes) && hoistable(no),
            _ => false,
        })
    }
    fn expressions<'a>(body: &'a [Statement], found: &mut Vec<&'a str>) {
        for statement in body {
            match statement {
                Statement::Return(Some(value)) => found.push(&value.cpp),
                Statement::Evaluate(text) => found.push(text),
                Statement::Branch { condition, yes, no } => {
                    found.push(&condition.cpp);
                    expressions(yes, found);
                    expressions(no, found);
                }
                _ => {}
            }
        }
    }
    /// Every complete call expression in `text`, including nested ones.
    fn calls(text: &str, found: &mut BTreeSet<String>) {
        for (start, _) in text.match_indices(CALL) {
            let mut depth = 0usize;
            for (offset, byte) in text.as_bytes()[start..].iter().enumerate() {
                match byte {
                    b'"' | b'\'' => break,
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            found.insert(text[start..=start + offset].to_string());
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    /// Evaluated whenever `text` is, and how many times `text` spells it.
    fn uses_text(text: &str, key: &str) -> (bool, usize) {
        let count = text.matches(key).count();
        let unconditional = text.find(key).is_some_and(|first| {
            let before = &text[..first];
            !before.contains("&&") && !before.contains("||") && !before.contains('?')
        });
        (unconditional, count)
    }
    /// Evaluated on every path through `body`, and the most on any one path.
    fn uses(body: &[Statement], key: &str) -> (bool, usize) {
        let Some((first, rest)) = body.split_first() else {
            return (false, 0);
        };
        match first {
            Statement::Return(Some(value)) => uses_text(&value.cpp, key),
            Statement::Return(None) => (false, 0),
            Statement::Evaluate(text) => {
                let (must, count) = uses_text(text, key);
                let (later_must, later) = uses(rest, key);
                (must || later_must, count + later)
            }
            Statement::Branch { condition, yes, no } => {
                let (must, count) = uses_text(&condition.cpp, key);
                let path = |arm: &[Statement]| {
                    let mut steps = arm.to_vec();
                    steps.extend_from_slice(rest);
                    uses(&steps, key)
                };
                let (yes_must, yes_count) = path(yes);
                let (no_must, no_count) = path(no);
                (
                    must || (yes_must && no_must),
                    count + yes_count.max(no_count),
                )
            }
            _ => uses(rest, key),
        }
    }
    fn replace(body: &mut [Statement], key: &str, local: &str) {
        for statement in body {
            match statement {
                Statement::Return(Some(value)) => value.cpp = value.cpp.replace(key, local),
                Statement::Evaluate(text) => *text = text.replace(key, local),
                Statement::Branch { condition, yes, no } => {
                    condition.cpp = condition.cpp.replace(key, local);
                    replace(yes, key, local);
                    replace(no, key, local);
                }
                _ => {}
            }
        }
    }
    fn hoist(mut body: Vec<Statement>, serial: &mut usize) -> Vec<Statement> {
        'search: loop {
            let mut found = vec![];
            expressions(&body, &mut found);
            let mut keys = BTreeSet::new();
            for text in found {
                calls(text, &mut keys);
            }
            let mut keys = keys.into_iter().collect::<Vec<_>>();
            keys.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
            for key in keys {
                for index in 0..body.len() {
                    let (must, count) = uses(&body[index..], &key);
                    if must && count >= 2 {
                        *serial += 1;
                        let local = format!("epok_pure_{serial}");
                        replace(&mut body[index..], &key, &local);
                        body.insert(
                            index,
                            Statement::Evaluate(format!("const auto {local}={key}")),
                        );
                        continue 'search;
                    }
                }
            }
            break;
        }
        for statement in &mut body {
            if let Statement::Branch { yes, no, .. } = statement {
                *yes = hoist(std::mem::take(yes), serial);
                *no = hoist(std::mem::take(no), serial);
            }
        }
        body
    }
    if !hoistable(&body) {
        return body;
    }
    let mut found = vec![];
    expressions(&body, &mut found);
    if found.iter().map(|text| text.len()).sum::<usize>() > MAXIMUM_BODY_BYTES {
        return body;
    }
    hoist(body, &mut 0)
}

#[cfg(test)]
mod hoist_tests {
    use super::*;
    const WORK: &str = "([&]() -> Fixed {auto a=(x);return this->work(a);}())";
    fn expr(cpp: &str) -> Expression {
        Expression {
            value_type: Type::Fixed,
            cpp: cpp.into(),
            lvalue: false,
        }
    }
    fn returning(cpp: String) -> Statement {
        Statement::Return(Some(expr(&cpp)))
    }

    #[test]
    fn repeated_call_on_one_path_evaluates_once() {
        let body = hoist_pure_calls(vec![returning(format!("add({WORK},{WORK})"))]);
        assert_eq!(body.len(), 2);
        let Statement::Evaluate(local) = &body[0] else {
            panic!("expected a local");
        };
        assert_eq!(local, &format!("const auto epok_pure_1={WORK}"));
        assert_eq!(body[1], returning("add(epok_pure_1,epok_pure_1)".into()));
    }

    #[test]
    fn one_use_on_each_path_stays_in_place() {
        let body = vec![Statement::Branch {
            condition: expr("flag"),
            yes: vec![returning(WORK.into())],
            no: vec![returning(WORK.into())],
        }];
        assert_eq!(hoist_pure_calls(body.clone()), body);
    }

    #[test]
    fn a_guard_keeps_protecting_its_call() {
        let body = hoist_pure_calls(vec![
            Statement::Branch {
                condition: expr("!ready"),
                yes: vec![returning("0".into())],
                no: vec![],
            },
            returning(format!("add({WORK},{WORK})")),
        ]);
        assert!(matches!(body[0], Statement::Branch { .. }));
        assert!(matches!(&body[1], Statement::Evaluate(_)));
    }

    #[test]
    fn a_short_circuit_keeps_protecting_its_call() {
        let body = vec![returning(format!("(ready && (({WORK})>0)) ? {WORK} : 0"))];
        assert_eq!(hoist_pure_calls(body.clone()), body);
    }

    #[test]
    fn unknown_statements_leave_the_function_untouched() {
        let body = vec![
            Statement::Evaluate("self_write()".into()),
            returning(format!("add({WORK},{WORK})")),
        ];
        assert_eq!(hoist_pure_calls(body.clone()), body);
    }

    #[test]
    fn nested_repetition_collapses_outermost_first() {
        let outer = format!("([&]() -> Fixed {{auto a=({WORK});return this->more(a);}}())");
        let body = hoist_pure_calls(vec![returning(format!("add({outer},{outer},{WORK})"))]);
        let text = format!("{body:?}");
        assert_eq!(text.matches("this->more").count(), 1);
        assert_eq!(text.matches("this->work").count(), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(ty: Type, cpp: &str) -> Expression {
        Expression {
            value_type: ty,
            cpp: cpp.into(),
            lvalue: false,
        }
    }

    /// Equal and Not equal take any two references and compare their identities;
    /// references have no order and never equal a number.
    #[test]
    fn references_compare_by_identity() {
        let other = || value(Type::ObjectRef { class: None }, "other");
        let player = || {
            value(
                Type::ActorRef {
                    class: Some(crate::object_model::ACTOR3D_ID.into()),
                },
                "player",
            )
        };
        let same = binary(BinaryOp::Equal, other(), player()).unwrap();
        assert_eq!(
            (same.value_type, same.cpp.as_str()),
            (Type::Bool, "((other)==(player))")
        );
        let different = binary(BinaryOp::NotEqual, player(), other()).unwrap();
        assert_eq!(different.cpp, "((player)!=(other))");
        assert!(binary(BinaryOp::Less, other(), player()).is_err());
        assert!(binary(BinaryOp::Equal, other(), value(Type::Int32, "1")).is_err());
    }
}

#[cfg(test)]
mod shared_tail_tests {
    use super::*;
    fn e(cpp: &str) -> Expression {
        Expression {
            value_type: Type::Int32,
            cpp: cpp.into(),
            lvalue: false,
        }
    }
    fn n(id: &str) -> Statement {
        Statement::Node(id.into())
    }
    fn set(target: &str, value: &str) -> Statement {
        Statement::Assign {
            target: target.into(),
            value: e(value),
        }
    }
    fn branch(yes: Vec<Statement>, no: Vec<Statement>) -> Statement {
        let mut condition = e("flag");
        condition.value_type = Type::Bool;
        Statement::Branch { condition, yes, no }
    }
    fn count(body: &[Statement], id: &str) -> usize {
        body.iter()
            .map(|s| match s {
                Statement::Node(value) => usize::from(value == id),
                Statement::Branch { yes, no, .. } => count(yes, id) + count(no, id),
                Statement::Loop { body, .. } => count(body, id),
                _ => 0,
            })
            .sum()
    }
    #[derive(Clone, Debug, PartialEq)]
    struct State {
        health: i32,
        old: i32,
        flag: bool,
        alive: bool,
        done: bool,
        returned: Option<i32>,
        calls: u32,
        trace: Vec<String>,
        conditions: u32,
    }
    impl State {
        fn new(flag: bool) -> Self {
            Self {
                health: 35,
                old: -1,
                flag,
                alive: true,
                done: false,
                returned: None,
                calls: 0,
                trace: vec![],
                conditions: 0,
            }
        }
        fn value(&mut self, value: &str) -> i32 {
            match value {
                "flag" => {
                    self.conditions += 1;
                    i32::from(self.flag)
                }
                "get_health()" => {
                    self.calls += 1;
                    self.health
                }
                "old" => self.old,
                "health" => self.health,
                "health+1" => self.health + 1,
                _ => value.parse().unwrap(),
            }
        }
    }
    fn execute(body: &[Statement], state: &mut State) {
        for statement in body {
            if state.done {
                return;
            }
            match statement {
                Statement::Node(id) => state.trace.push(id.clone()),
                Statement::Assign { target, value } => {
                    let value = state.value(&value.cpp);
                    match target.as_str() {
                        "health" => state.health = value,
                        "old" => state.old = value,
                        _ => panic!("unknown target"),
                    }
                }
                Statement::Evaluate(value) if value == "destroy_owner" => state.alive = false,
                Statement::CheckOwner(_) => {
                    if !state.alive {
                        state.done = true;
                    }
                }
                Statement::Branch { condition, yes, no } => {
                    let flag = state.value(&condition.cpp) != 0;
                    execute(if flag { yes } else { no }, state);
                }
                Statement::Loop { count, body } => {
                    for _ in 0..*count {
                        execute(body, state);
                        if state.done {
                            break;
                        }
                    }
                }
                Statement::Return(value) => {
                    state.returned = value.as_ref().map(|value| state.value(&value.cpp));
                    state.done = true;
                }
                _ => panic!("non-synchronous fixture"),
            }
        }
    }
    fn same(body: Vec<Statement>) -> Vec<Statement> {
        let result = factor_shared_branch_tails(body.clone());
        for flag in [false, true] {
            let mut a = State::new(flag);
            let mut b = a.clone();
            execute(&body, &mut a);
            execute(&result, &mut b);
            assert_eq!(a, b);
        }
        result
    }
    #[test]
    fn forward_shared_join_is_emitted_once_with_trace_and_effects() {
        let tail = vec![
            n("join"),
            set("health", "health+1"),
            n("return"),
            Statement::Return(Some(e("health"))),
        ];
        let mut yes = vec![n("yes"), set("health", "1")];
        yes.extend(tail.clone());
        let mut no = vec![n("no"), set("health", "2")];
        no.extend(tail);
        let body = vec![n("branch"), branch(yes, no)];
        assert_eq!(count(&body, "join"), 2);
        let result = same(body);
        assert_eq!(count(&result, "join"), 1);
        assert!(matches!(result[2],Statement::Node(ref id) if id=="join"));
    }
    #[test]
    fn explicit_captured_getter_stays_before_write_and_old_value_use() {
        let tail = vec![n("join"), Statement::Return(Some(e("old")))];
        let mut yes = vec![n("write-zero"), set("health", "0")];
        yes.extend(tail.clone());
        let mut no = vec![n("write-one"), set("health", "1")];
        no.extend(tail);
        let result = same(vec![
            n("read"),
            set("old", "get_health()"),
            n("branch"),
            branch(yes, no),
        ]);
        for flag in [false, true] {
            let mut state = State::new(flag);
            execute(&result, &mut state);
            assert_eq!(state.returned, Some(35));
            assert_eq!(state.calls, 1);
        }
    }
    #[test]
    fn branch_local_capture_is_not_moved_before_its_mutation() {
        let tail = vec![
            n("read-shared"),
            set("old", "get_health()"),
            n("return"),
            Statement::Return(Some(e("old"))),
        ];
        let mut yes = vec![n("set0"), set("health", "0")];
        yes.extend(tail.clone());
        let mut no = vec![n("set1"), set("health", "1")];
        no.extend(tail);
        let result = same(vec![branch(yes, no)]);
        for flag in [false, true] {
            let mut state = State::new(flag);
            execute(&result, &mut state);
            assert_eq!(state.returned, Some(if flag { 0 } else { 1 }));
            assert_eq!(state.calls, 1);
        }
    }
    #[test]
    fn different_capture_expression_or_node_origin_is_never_merged() {
        let body = vec![branch(
            vec![n("same"), set("old", "health")],
            vec![n("same"), set("old", "get_health()")],
        )];
        assert_eq!(factor_shared_branch_tails(body.clone()), body);
        let body = vec![branch(
            vec![n("different-yes"), set("old", "get_health()")],
            vec![n("different-no"), set("old", "get_health()")],
        )];
        assert_eq!(factor_shared_branch_tails(body.clone()), body);
    }
    #[test]
    fn early_return_owner_guard_and_loop_exit_preserve_path_order() {
        let body = vec![
            Statement::Loop {
                count: 4,
                body: vec![branch(
                    vec![n("return-early"), Statement::Return(Some(e("health")))],
                    vec![n("continue"), set("health", "health+1")],
                )],
            },
            n("end"),
            Statement::Return(Some(e("health"))),
        ];
        same(body);
        let tail = vec![
            n("guard"),
            Statement::CheckOwner(Type::Void),
            n("after"),
            set("health", "0"),
        ];
        let mut yes = vec![n("destroy"), Statement::Evaluate("destroy_owner".into())];
        yes.extend(tail.clone());
        same(vec![branch(yes, tail)]);
    }
    #[test]
    fn nested_diamonds_and_loop_body_do_not_duplicate_effects() {
        let mut tail = vec![n("last"), set("health", "health+1")];
        for index in 0..7 {
            let mut yes = vec![n(&format!("y{index}"))];
            yes.extend(tail.clone());
            let mut no = vec![n(&format!("n{index}"))];
            no.extend(tail);
            tail = vec![n(&format!("b{index}")), branch(yes, no)];
        }
        assert_eq!(count(&tail, "last"), 128);
        let factored = same(vec![Statement::Loop {
            count: 3,
            body: tail,
        }]);
        assert_eq!(count(&factored, "last"), 1);
    }
    #[test]
    fn condition_is_not_elided_when_its_continuation_is_shared() {
        let result = same(vec![branch(
            vec![n("join"), set("health", "0")],
            vec![n("join"), set("health", "0")],
        )]);
        assert!(
            matches!(result[0],Statement::Branch {ref yes,ref no,..} if yes.is_empty()&&no.is_empty())
        );
        for flag in [false, true] {
            let mut state = State::new(flag);
            execute(&result, &mut state);
            assert_eq!(state.conditions, 1);
        }
    }
    #[test]
    fn latent_delay_timeline_wait_and_break_machine_remain_untouched() {
        let timeline = Statement::Timeline {
            node: "timeline".into(),
            target: "x".into(),
            keys: vec![],
            looping: false,
            updated: vec![],
            finished: vec![],
        };
        let wait = Statement::WaitPlayback {
            node: "wait".into(),
            repeating: false,
            target: e("0"),
            asset: 1,
            marker: 2,
            reached: vec![],
            completed: vec![],
            cancelled: vec![],
        };
        for latent in [
            Statement::Delay(e("1")),
            timeline,
            wait,
            Statement::StopTimeline("x".into()),
        ] {
            let body = vec![branch(
                vec![n("join"), latent.clone()],
                vec![n("join"), latent],
            )];
            assert_eq!(
                factor_shared_branch_tails(body.clone()),
                body,
                "latent state-machine break/resume scopes cannot be factored"
            );
        }
    }
}
