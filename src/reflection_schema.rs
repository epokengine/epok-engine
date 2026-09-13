//! The versioned wire contract between the host extractor and the editor.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

pub const SCHEMA_VERSION: u32 = 8;
pub const CLANG_VERSION: &str = "18.1.1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Type {
    Void,
    Bool,
    Int32,
    #[serde(rename = "uint32")]
    UInt32,
    #[default]
    Fixed,
    Enum {
        cpp_name: String,
        variants: BTreeMap<String, i64>,
    },
    /// A bounded Fixed array is the initial native representation for vectors.
    Vector {
        length: usize,
    },
    /// Lifecycle records and reflected plain native structs; never native pointers.
    Record {
        cpp_name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fields: Vec<RecordField>,
    },
    /// Persistent authoring UUID, resolved to a generation-checked handle before start.
    EntityRef {
        class: Option<String>,
    },
    /// Internal ParticleEffect layer UUID. Never a scene entity or runtime slot.
    EffectLayerRef {
        class: String,
    },
    /// Transient playback results. Only null defaults may be authored.
    SequenceHandle,
    EffectHandle,
    AssetRef {
        #[serde(rename = "asset_kind")]
        kind: String,
    },
    ClassRef {
        base: String,
    },
    /// Persistent authoring UUID of an `epok::Object` instance (schema 8).
    /// `class` narrows the accepted class; `None` accepts any reflected object.
    ObjectRef {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        class: Option<String>,
    },
    /// Persistent authoring UUID of an Actor instance (schema 8).
    ActorRef {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        class: Option<String>,
    },
    /// Persistent authoring UUID of an ActorComponent instance (schema 8).
    ComponentRef {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        class: Option<String>,
    },
}

/// Family root of a reflected class. A parent change never crosses families.
/// `Behaviour` is the legacy family: every pre-schema-8 script class lands here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassFamily {
    Object,
    Actor,
    Component,
    World,
    Level,
    #[default]
    Behaviour,
}
impl ClassFamily {
    #[allow(dead_code)] // Shared with the extraction binary, which only writes the value.
    pub fn label(self) -> &'static str {
        match self {
            Self::Object => "Object",
            Self::Actor => "Actor",
            Self::Component => "Component",
            Self::World => "World",
            Self::Level => "Level",
            Self::Behaviour => "Behaviour",
        }
    }
}

/// Spatial/layout domain. Derived from the class; never stored as scene truth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    #[default]
    None,
    #[serde(rename = "world3d")]
    World3D,
    #[serde(rename = "world2d")]
    World2D,
    #[serde(rename = "ui")]
    UI,
}
impl Domain {
    #[allow(dead_code)] // Shared with the extraction binary.
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::World3D => "World3D",
            Self::World2D => "World2D",
            Self::UI => "UI",
        }
    }
    /// Domains that own a spatial/layout root component.
    #[allow(dead_code)] // Shared with the extraction binary.
    pub fn spatial(self) -> bool {
        self != Self::None
    }
}

/// How an actor class may enter a level. `scene_managed` classes are created by
/// the level loader only and are never placeable or spawnable.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Placement {
    #[serde(default)]
    pub placeable: bool,
    #[serde(default)]
    pub spawnable: bool,
    #[serde(default)]
    pub scene_managed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    #[default]
    Single,
    Multiple,
}

/// What a component class demands of the actor that owns it. Declared tokens are
/// merged with the inherited contract by `object_model::Model::from_registry`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentContract {
    /// Actor domains allowed to own this component. Empty = inherit (all domains
    /// at the family root). A descendant may only narrow this set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub owners: BTreeSet<Domain>,
    /// `cpp_name` of component classes that must exist on the same owner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub cardinality: Cardinality,
    /// May act as the actor's spatial/layout root.
    #[serde(default)]
    pub can_root: bool,
    /// Target capabilities the cooked build must provide.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub capabilities: BTreeSet<String>,
}

/// A component instance declared on a class by an `EPOK_COMPONENT` field.
/// Defaults stay declarative and inspectable; constructors remain defaulted.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultComponent {
    pub id: String,
    /// Name of the annotated C++ field.
    pub field: String,
    /// `cpp_name` of the component class.
    pub class: String,
    #[serde(default)]
    pub root: bool,
    /// Field name of the default component this one attaches to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub value_type: Type,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Value,
    ConstReference,
    MutableReference,
}
impl Type {
    /// Value members only. Handles and opaque runtime records are intentionally
    /// not decomposed into their internal indices or pointers.
    #[allow(dead_code)] // Shared with the extraction binary, which only writes fields.
    pub fn members(&self) -> Vec<RecordField> {
        match self {
            Self::Vector {
                length: length @ 2..=3,
            } => ["x", "y", "z"]
                .iter()
                .take(*length)
                .map(|name| RecordField {
                    name: (*name).into(),
                    value_type: Self::Fixed,
                })
                .collect(),
            Self::Record { cpp_name, .. } if cpp_name == "epok::Transform" => {
                ["position", "rotation", "scale"]
                    .iter()
                    .map(|name| RecordField {
                        name: (*name).into(),
                        value_type: Self::Vector { length: 3 },
                    })
                    .collect()
            }
            Self::Record { fields, .. } => fields.clone(),
            _ => vec![],
        }
    }
    #[allow(dead_code)] // Shared with the extraction binary.
    pub fn member_type(&self, path: &str) -> Option<Type> {
        if path.is_empty() {
            return Some(self.clone());
        }
        let mut ty = self.clone();
        for name in path.split('.').take(17) {
            ty = ty
                .members()
                .into_iter()
                .find(|f| f.name == name)?
                .value_type;
        }
        if path.split('.').count() > 16 {
            None
        } else {
            Some(ty)
        }
    }
    #[allow(dead_code)] // The host extractor shares the wire schema, but does not render UI labels.
    pub fn label(&self) -> String {
        match self {
            Self::Void => "void".into(),
            Self::Bool => "bool".into(),
            Self::Int32 => "int32".into(),
            Self::UInt32 => "uint32".into(),
            Self::Fixed => "Fixed (Q12)".into(),
            Self::Enum { cpp_name, .. } | Self::Record { cpp_name, .. } => cpp_name.clone(),
            Self::Vector { length } => format!("Fixed[{length}]"),
            Self::EntityRef { class } => {
                format!("EntityRef<{}>", class.as_deref().unwrap_or("Entity"))
            }
            Self::EffectLayerRef { class } => format!("EffectLayerRef<{class}>"),
            Self::SequenceHandle => "SequenceHandle".into(),
            Self::EffectHandle => "EffectHandle".into(),
            Self::AssetRef { kind } => format!("AssetRef<{kind}>"),
            Self::ClassRef { base } => format!("ClassRef<{base}>"),
            Self::ObjectRef { class } => {
                format!("ObjectRef<{}>", class.as_deref().unwrap_or("Object"))
            }
            Self::ActorRef { class } => {
                format!("ActorRef<{}>", class.as_deref().unwrap_or("Actor"))
            }
            Self::ComponentRef { class } => format!(
                "ComponentRef<{}>",
                class.as_deref().unwrap_or("ActorComponent")
            ),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value_type: Type,
    pub direction: Direction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub name: String,
    pub value_type: Type,
    pub default: serde_json::Value,
    pub editable: bool,
    /// Explicit opt-in; old manifests never gain animation permission on load.
    #[serde(default)]
    pub timeline: Option<TimelineProperty>,
    pub source: Location,
}

/// Explicit timeline capability profiles; editability alone grants no permission.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimelineProperty {
    /// The v2 prototype remains readable without broadening its permissions.
    FixedLinearAbsolute,
    NativeField {
        interpolation: Vec<Interpolation>,
        blends: Vec<Blend>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Interpolation {
    Linear,
    Step,
    Smoothstep,
    EaseIn,
    EaseOut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Blend {
    Absolute,
    Additive,
}
impl TimelineProperty {
    pub fn for_type(ty: &Type) -> Option<Self> {
        let numeric = matches!(
            ty,
            Type::Fixed | Type::Int32 | Type::UInt32 | Type::Vector { length: 2..=3 }
        );
        let discrete = matches!(ty, Type::Bool)
            || matches!(ty,Type::Enum{variants,..} if variants.values().all(|v|i32::try_from(*v).is_ok()));
        if !numeric && !discrete {
            return None;
        }
        Some(Self::NativeField {
            interpolation: if numeric {
                vec![
                    Interpolation::Linear,
                    Interpolation::Step,
                    Interpolation::Smoothstep,
                    Interpolation::EaseIn,
                    Interpolation::EaseOut,
                ]
            } else {
                vec![Interpolation::Step]
            },
            blends: if numeric {
                vec![Blend::Absolute, Blend::Additive]
            } else {
                vec![Blend::Absolute]
            },
        })
    }
    #[allow(dead_code)] // The extractor emits profiles; host cooking consumes permissions.
    pub fn allows(&self, mode: Interpolation, blend: Blend) -> bool {
        match self {
            Self::FixedLinearAbsolute => mode == Interpolation::Linear && blend == Blend::Absolute,
            Self::NativeField {
                interpolation,
                blends,
            } => interpolation.contains(&mode) && blends.contains(&blend),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineCall {
    CrossingEvent,
    IdempotentAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub id: String,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub returns: Type,
    pub callable: bool,
    #[serde(default)]
    pub timeline: Option<TimelineCall>,
    pub event: bool,
    pub pure: bool,
    pub abstract_method: bool,
    pub final_method: bool,
    pub access: String,
    pub overrides: Vec<String>,
    pub source: Location,
}

/// A reflected scene adapter can require one existing component on its owner.
/// This narrows EntityRef compatibility; it introduces no new entity identity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimelineComponentRequirement {
    Camera,
    AudioSource,
    Light,
    PaletteAnimator,
    ParticleEmitter,
    RectTransform,
    Text,
    Image,
    ProgressBar,
}
impl TimelineComponentRequirement {
    #[allow(dead_code)] // The header extractor shares this schema, not code generation.
    pub fn runtime_member(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::AudioSource => "audio.enabled",
            Self::Light => "light.enabled",
            Self::PaletteAnimator => "palette_animator.enabled",
            Self::ParticleEmitter => "particle_emitter.enabled",
            Self::RectTransform => "rect.enabled",
            Self::Text => "text.enabled",
            Self::Image => "image.enabled",
            Self::ProgressBar => "progress.enabled",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    #[serde(default = "native_provider")]
    pub provider: Extension,
    #[serde(default = "native_backend")]
    pub backend: Extension,
    pub cpp_name: String,
    pub parent: Option<String>,
    pub abstract_class: bool,
    pub final_class: bool,
    pub blueprintable: bool,
    #[serde(default)]
    pub timeline_component: Option<TimelineComponentRequirement>,
    /// Declared family; `None` inherits the parent's. Schema 8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<ClassFamily>,
    /// Declared domain; `None` inherits the parent's. Schema 8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,
    #[serde(default)]
    pub placement: Placement,
    /// Declared component contract; `None` inherits the parent's. Schema 8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<ComponentContract>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_components: Vec<DefaultComponent>,
    /// `Abstract` token; `is_abstract_record` sets `abstract_class` independently.
    #[serde(default)]
    pub explicit_abstract: bool,
    pub properties: Vec<Property>,
    pub functions: Vec<Function>,
    pub source: Location,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extension {
    pub id: String,
    pub version: u32,
}
pub fn native_provider() -> Extension {
    Extension {
        id: "cpp".into(),
        version: 1,
    }
}
pub fn native_backend() -> Extension {
    Extension {
        id: "native".into(),
        version: 1,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub clang_version: String,
    pub target: String,
    pub classes: Vec<Class>,
    /// Every header Clang actually included, including SDK and standard headers.
    pub dependencies: BTreeMap<PathBuf, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub source: PathBuf,
    pub arguments: Vec<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_seven_classes_deserialize_without_actor_metadata() {
        let legacy = serde_json::json!({
            "id":"8ec3a9d4-13f1-4727-b4b1-591202c82490", "cpp_name":"epok::Behaviour",
            "parent":null, "abstract_class":false, "final_class":false, "blueprintable":true,
            "properties":[], "functions":[],
            "source":{"file":"epok.hpp","line":1,"column":1}
        });
        let class: super::Class = serde_json::from_value(legacy).unwrap();
        assert_eq!(class.family, None);
        assert_eq!(class.domain, None);
        assert_eq!(class.placement, super::Placement::default());
        assert!(class.component.is_none() && class.default_components.is_empty());
        assert!(!class.explicit_abstract);
        // The defaults never appear in the wire form of an untouched class.
        let round_trip = serde_json::to_value(&class).unwrap();
        for field in ["family", "domain", "component", "default_components"] {
            assert!(round_trip.get(field).is_none(), "{field} was serialized");
        }
        assert_eq!(super::SCHEMA_VERSION, 8);
    }

    #[test]
    fn typed_object_references_label_their_class() {
        use super::Type;
        let actor = Type::ActorRef {
            class: Some("epok::Actor3D".into()),
        };
        assert_eq!(actor.label(), "ActorRef<epok::Actor3D>");
        assert_eq!(Type::ObjectRef { class: None }.label(), "ObjectRef<Object>");
        assert_eq!(
            Type::ComponentRef { class: None }.label(),
            "ComponentRef<ActorComponent>"
        );
        // Typed references are opaque identities, never decomposed into members.
        assert!(actor.members().is_empty());
        let restored: Type = serde_json::from_slice(&serde_json::to_vec(&actor).unwrap()).unwrap();
        assert_eq!(restored, actor);
    }

    #[test]
    fn legacy_editability_never_grants_timeline_access() {
        let legacy = serde_json::json!({
            "id":"d2928b3f-7cdd-4e11-9b5b-3e37b69baf8a", "name":"progress",
            "value_type":{"kind":"fixed"}, "default":0, "editable":true,
            "source":{"file":"Probe.hpp","line":1,"column":1}
        });
        let property: super::Property = serde_json::from_value(legacy).unwrap();
        assert!(property.timeline.is_none());
        let mut exposed = property;
        exposed.timeline = Some(super::TimelineProperty::FixedLinearAbsolute);
        let restored: super::Property =
            serde_json::from_slice(&serde_json::to_vec(&exposed).unwrap()).unwrap();
        assert_eq!(restored.timeline, exposed.timeline);
        assert_eq!(restored.id, exposed.id);
    }
}
