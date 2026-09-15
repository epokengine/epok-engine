//! Host-side single source of truth for the Object / Actor / Component model.
//!
//! `Model::from_registry` resolves the declared reflection metadata (schema 8)
//! into fully inherited class records and enforces the rules of
//! `knowledge/initiatives/actor-architecture/design.md` sections 2 and 5. Every
//! consumer — component picker, Inspector, scene load, Blueprint compiler,
//! exporter, C++ API generation, MCP — asks these functions instead of
//! re-implementing the compatibility rules.
//!
//! P1 introduces the model and its contracts ahead of the consumers that call
//! them (Inspector, scene load, exporter, MCP), so the public surface is not yet
//! referenced outside the tests.
#![allow(dead_code)]
use crate::{
    blueprint,
    reflection_schema::{
        self as schema, Cardinality, ClassFamily, ComponentContract, DefaultComponent, Domain,
        Extension, Placement,
    },
    script_backend,
};
use std::collections::{BTreeMap, BTreeSet};

// Stable identities of the native bases (design.md section 2). A unit test in
// this module checks each one against `runtime/object_model.hpp`.
pub const OBJECT_ID: &str = "26a54c0d-ca81-41ca-aecc-d0346a6357d2";
pub const ACTOR_ID: &str = "6e6efc67-66c4-4dae-90f8-7c8c4e612dea";
pub const ACTOR3D_ID: &str = "fc24ce9b-558c-49de-bc35-e040f350e486";
pub const ACTOR2D_ID: &str = "5308054e-0aaa-4d53-963b-440cf0c71916";
pub const UI_ACTOR_ID: &str = "b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f";
pub const SCENE_SCRIPT_ACTOR_ID: &str = "b4c08aa0-fa85-4abf-8f45-7501e1c8a040";
pub const ACTOR_COMPONENT_ID: &str = "2e5021ee-d14d-4d77-9112-455f29d639d2";
pub const SCENE_COMPONENT3D_ID: &str = "ed73d249-b6cb-4a3c-a0e8-696de55e286f";
pub const SCENE_COMPONENT2D_ID: &str = "27887770-a779-4a16-863c-5abd32786cfa";
pub const UI_COMPONENT_ID: &str = "83bffb60-2c33-4be1-9041-8c8f4c395b86";
pub const RECT_TRANSFORM_COMPONENT_ID: &str = "dc805165-6c65-48dc-8ff8-4a638a5d21df";
pub const AUDIO_COMPONENT_ID: &str = "7f0eb028-5301-4ac7-b93b-5665fab12b20";
pub const WORLD_ID: &str = "ab2d72b4-fcf6-4b30-8494-43fc0a7cf46c";
pub const LEVEL_ID: &str = "b4683321-83e2-4b90-bd87-bb314f9eda2e";

// Reserved for the P3/P8 component adapters; declared here so the identities are
// allocated once and cannot drift when the classes are introduced.
pub const MESH3D_COMPONENT_ID: &str = "580f99b1-c905-4f96-b34f-807c51335ba0";
pub const SPRITE3D_COMPONENT_ID: &str = "0b68656b-364b-441f-ad86-ddc0408e80a0";
pub const CAMERA3D_COMPONENT_ID: &str = "9fe2abff-f285-435d-976d-825c5db5420a";
pub const LIGHT3D_COMPONENT_ID: &str = "e18d296c-5820-4557-9386-8b12b9ca37a2";
pub const COLLIDER3D_COMPONENT_ID: &str = "e8431f94-e526-4d7d-aace-8c8fae7955e6";
pub const SPRITE2D_COMPONENT_ID: &str = "892ac5c9-eae8-48b8-853a-db93d20a537b";
pub const CAMERA2D_COMPONENT_ID: &str = "7245e90e-d900-4976-b738-0f95caa1d15d";
pub const COLLIDER2D_COMPONENT_ID: &str = "aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe";
pub const CANVAS_COMPONENT_ID: &str = "f2cfb26b-af53-4a46-9d91-1debea72e01b";
pub const IMAGE_COMPONENT_ID: &str = "d04c78d6-23bd-40d7-88f1-b1afc1b05b5b";
pub const TEXT_COMPONENT_ID: &str = "fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34";
pub const PROGRESS_BAR_COMPONENT_ID: &str = "28bf5245-5d80-4cba-a77d-1d74479ac276";

/// Capability names the PSX target provides unconditionally. A component may
/// declare any name (design.md section 3); one that the target does not provide
/// is a build diagnostic carrying the name, never a silent success.
const PSX_CAPABILITIES: &[&str] = &[
    "audio",
    "hud",
    "sprites",
    "particles",
    "timelines",
    "physics3d",
    "mesh",
    "sprite",
    "camera",
    "light",
    "collider",
    "canvas",
    "image",
    "text",
    "progress",
    "timeline",
    "effect",
    "palette",
    "shadow",
];
/// Provided only when `runtime/world2d.hpp` is part of the cooked runtime source
/// set; the 2D collision world lives entirely in that header.
const PHYSICS2D_CAPABILITY: &str = "physics2d";

/// What the build target can actually provide. Components declare `Capability=`
/// tokens; `Model::validate_capabilities` compares the two and reports the gap.
/// Built from the project/target rather than hard-coded so a future target (or a
/// runtime trimmed of a subsystem) narrows the set in one place.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TargetCapabilities {
    names: BTreeSet<String>,
}
impl TargetCapabilities {
    /// The PSX runtime as this repository cooks it.
    pub fn psx() -> Self {
        // `runtime_sources` is the exact list the cook copies into the build, so
        // asking it is the same question as "is world2d.hpp present".
        Self::psx_with_world2d(
            crate::project::runtime_sources()
                .iter()
                .any(|(name, _)| *name == "world2d.hpp"),
        )
    }
    /// Explicit form, for tests and for a target that trims the 2D world.
    pub fn psx_with_world2d(world2d: bool) -> Self {
        let mut names: BTreeSet<String> =
            PSX_CAPABILITIES.iter().map(|s| (*s).to_string()).collect();
        if world2d {
            names.insert(PHYSICS2D_CAPABILITY.to_string());
        }
        Self { names }
    }
    /// A target that provides nothing; every `Capability=` token is a diagnostic.
    pub fn none() -> Self {
        Self::default()
    }
    pub fn with(mut self, name: &str) -> Self {
        self.names.insert(name.to_string());
        self
    }
    pub fn without(mut self, name: &str) -> Self {
        self.names.remove(name);
        self
    }
    pub fn provides(&self, name: &str) -> bool {
        self.names.contains(name)
    }
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(String::as_str)
    }
}

/// The component class that satisfies a timeline's `TimelineRequires=` token.
///
/// The timeline adapters address legacy entity members (`ComponentRequirement::
/// runtime_member`); the same requirement expressed on an actor is satisfied by
/// the component class reserved for it in design.md section 2. `PaletteAnimator`
/// and `ParticleEmitter` have no reserved actor component yet, so they return
/// `None` and keep their legacy-entity-only enforcement; introducing those two
/// classes is the only change needed to complete the mapping.
pub fn timeline_requirement_component(
    requirement: schema::TimelineComponentRequirement,
) -> Option<&'static str> {
    use schema::TimelineComponentRequirement as R;
    Some(match requirement {
        R::Camera => "epok::Camera3DComponent",
        R::AudioSource => "epok::AudioComponent",
        R::Light => "epok::Light3DComponent",
        R::RectTransform => "epok::RectTransformComponent",
        R::Text => "epok::TextComponent",
        R::Image => "epok::ImageComponent",
        R::ProgressBar => "epok::ProgressBarComponent",
        R::PaletteAnimator | R::ParticleEmitter => return None,
    })
}

/// A resolved class. Every field is the inherited, validated value, never the
/// raw declaration: consumers must not walk the parent chain themselves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassModel {
    pub id: String,
    pub cpp_name: String,
    pub parent: Option<String>,
    pub family: ClassFamily,
    pub domain: Domain,
    pub placement: Placement,
    /// Resolved contract for `ClassFamily::Component`; `None` for every other family.
    pub component: Option<ComponentContract>,
    pub abstract_class: bool,
    pub blueprintable: bool,
    pub final_class: bool,
    pub default_components: Vec<DefaultComponent>,
    pub provider: Extension,
    pub backend: Extension,
    /// Class ids from the root of the chain to this class inclusive. Abstract,
    /// non-instantiable bases stay in the table.
    pub ancestry: Vec<String>,
}
impl ClassModel {
    /// Whether `author` may declare a subclass of this class. Mirrors
    /// `script_backend::can_derive` over resolved values.
    pub fn derivable_by(&self, author: &Extension) -> bool {
        script_backend::capabilities(author)
            .is_ok_and(|c| c.create && c.derive_backends.contains(&self.backend.id))
            && (self.provider == schema::native_provider()
                || (*author == script_backend::blueprint_provider()
                    && self.provider == script_backend::blueprint_provider()))
            && self.backend == schema::native_backend()
            && self.blueprintable
            && !self.final_class
    }
    /// Instantiable classes only; abstract bases exist to be derived from.
    pub fn instantiable(&self) -> bool {
        !self.abstract_class
    }
}

/// How a script body reaches the transform of the actor it belongs to. It is
/// the same root-component transform the Blueprint Get/Set Position, Rotation
/// and Scale nodes address, so both authoring surfaces stay equivalent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransformAccess {
    /// `World3D` (Vector{3} position/rotation/scale) or `World2D`
    /// (Vector{2} position/scale, scalar `Fixed` rotation).
    pub domain: Domain,
    /// Components address their owner; actors address themselves.
    pub through_owner: bool,
}

/// Family, domain and component `Owners` of `cpp_name`, inherited the way
/// `Model::from_registry` inherits them but without resolving the whole graph:
/// the nearest declaring ancestor wins for each.
pub fn resolved_shape(
    registry: &blueprint::Registry,
    cpp_name: &str,
) -> (ClassFamily, Domain, BTreeSet<Domain>) {
    let (mut family, mut domain) = (ClassFamily::Object, Domain::None);
    let mut owners = BTreeSet::new();
    for class in registry.ancestry(cpp_name) {
        if let Some(declared) = class.family {
            family = declared;
        }
        if let Some(declared) = class.domain {
            domain = declared;
        }
        if let Some(contract) = &class.component
            && !contract.owners.is_empty()
        {
            owners = contract.owners.clone();
        }
    }
    (family, domain, owners)
}

/// Resolves `cpp_name`'s transform access straight from the registry instead of
/// building a whole `Model`: family, domain and the component `Owners` set are
/// inherited, so the nearest declaring ancestor wins. A component has no
/// transform of its own — its `Owners` contract says which actor domains may
/// carry it, and `World3D` wins when a contract admits both.
pub fn transform_access(registry: &blueprint::Registry, cpp_name: &str) -> Option<TransformAccess> {
    let (family, domain, owners) = resolved_shape(registry, cpp_name);
    let (domain, through_owner) = match family {
        ClassFamily::Actor => (domain, false),
        ClassFamily::Component if owners.contains(&Domain::World3D) => (Domain::World3D, true),
        ClassFamily::Component if owners.contains(&Domain::World2D) => (Domain::World2D, true),
        _ => return None,
    };
    matches!(domain, Domain::World3D | Domain::World2D).then_some(TransformAccess {
        domain,
        through_owner,
    })
}

/// One authored component on an actor instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentSpec {
    pub id: uuid::Uuid,
    /// Class id or `cpp_name`.
    pub class: String,
    pub root: bool,
}

/// A model or validation failure. `code` is stable for tests and MCP; `message`
/// is the human sentence; `class` names the class the failure belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub class: Option<String>,
}
impl Diagnostic {
    fn new(code: &'static str, class: Option<&str>, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            class: class.map(str::to_owned),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    classes: BTreeMap<String, ClassModel>,
    by_name: BTreeMap<String, String>,
}

/// Resolution state of one class while `from_registry` walks the graph.
enum Resolution {
    Pending,
    Done,
    Failed,
}

impl Model {
    /// Resolves inheritance for every class in `registry`.
    ///
    /// Returns every diagnostic found rather than the first: the caller shows a
    /// list. A class that produced a diagnostic is omitted from the model, so a
    /// partially broken project still yields a usable model for the rest.
    pub fn from_registry(registry: &blueprint::Registry) -> Result<Model, Vec<Diagnostic>> {
        let mut model = Model::default();
        let mut diagnostics = Vec::new();
        let mut state: BTreeMap<String, Resolution> = BTreeMap::new();
        for id in registry.classes.keys() {
            resolve(registry, id, &mut model, &mut state, &mut diagnostics);
        }
        if diagnostics.is_empty() {
            model.by_name = model
                .classes
                .values()
                .map(|c| (c.cpp_name.clone(), c.id.clone()))
                .collect();
            Ok(model)
        } else {
            Err(diagnostics)
        }
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &ClassModel> {
        self.classes.values()
    }
    /// Looks a class up by reflection id first, then by `cpp_name`.
    pub fn class(&self, id_or_cpp_name: &str) -> Option<&ClassModel> {
        self.classes.get(id_or_cpp_name).or_else(|| {
            self.by_name
                .get(id_or_cpp_name)
                .and_then(|id| self.classes.get(id))
        })
    }
    /// `true` when `class` is `base` or derives from it. Unknown names are `false`.
    pub fn is_a(&self, class: &str, base: &str) -> bool {
        let (Some(class), Some(base)) = (self.class(class), self.class(base)) else {
            return false;
        };
        class.ancestry.contains(&base.id)
    }
    /// Classes `author` may derive from within `family`, in `cpp_name` order.
    pub fn eligible_parents(&self, author: &Extension, family: ClassFamily) -> Vec<&ClassModel> {
        let mut parents = self
            .classes
            .values()
            .filter(|c| c.family == family && c.derivable_by(author))
            .collect::<Vec<_>>();
        parents.sort_by(|a, b| a.cpp_name.cmp(&b.cpp_name));
        parents
    }
    /// Whether `class` may be reparented onto `new_parent`: same family, domain
    /// compatible, parent derivable by the class's own provider, no cycle.
    pub fn validate_reparent(&self, class: &str, new_parent: &str) -> Result<(), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let (Some(class), Some(parent)) = (self.class(class), self.class(new_parent)) else {
            return Err(vec![Diagnostic::new(
                "unknown-class",
                None,
                format!("`{class}` or `{new_parent}` is not a reflected class"),
            )]);
        };
        if !parent.derivable_by(&class.provider) {
            diagnostics.push(Diagnostic::new(
                "parent-not-derivable",
                Some(&class.cpp_name),
                format!(
                    "`{}` cannot be a parent: it is not Blueprintable, is final, or belongs to another provider",
                    parent.cpp_name
                ),
            ));
        }
        if parent.family != class.family {
            diagnostics.push(Diagnostic::new(
                "family-crossing",
                Some(&class.cpp_name),
                format!(
                    "`{}` is a {} class; `{}` is a {} class. A parent change never crosses families.",
                    class.cpp_name,
                    class.family.label(),
                    parent.cpp_name,
                    parent.family.label()
                ),
            ));
        }
        if parent.domain != Domain::None
            && class.domain != Domain::None
            && parent.domain != class.domain
        {
            diagnostics.push(Diagnostic::new(
                "domain-mismatch",
                Some(&class.cpp_name),
                format!(
                    "`{}` is {}; `{}` is {}. The domain is inherited and cannot change.",
                    class.cpp_name,
                    class.domain.label(),
                    parent.cpp_name,
                    parent.domain.label()
                ),
            ));
        }
        if parent.id == class.id || parent.ancestry.contains(&class.id) {
            diagnostics.push(Diagnostic::new(
                "cyclic-parent",
                Some(&class.cpp_name),
                format!(
                    "`{}` already derives from `{}`; the change would create a cycle",
                    parent.cpp_name, class.cpp_name
                ),
            ));
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
    /// Whether one component class may be attached to one actor class.
    pub fn validate_component(
        &self,
        owner_class: &str,
        component_class: &str,
    ) -> Result<(), Diagnostic> {
        let owner = self.class(owner_class).ok_or_else(|| {
            Diagnostic::new(
                "unknown-class",
                None,
                format!("`{owner_class}` is not a reflected class"),
            )
        })?;
        let component = self.class(component_class).ok_or_else(|| {
            Diagnostic::new(
                "unknown-class",
                None,
                format!("`{component_class}` is not a reflected class"),
            )
        })?;
        if owner.family != ClassFamily::Actor {
            return Err(Diagnostic::new(
                "not-an-actor",
                Some(&owner.cpp_name),
                format!(
                    "`{}` is not an Actor; it owns no components",
                    owner.cpp_name
                ),
            ));
        }
        let Some(contract) = &component.component else {
            return Err(Diagnostic::new(
                "not-a-component",
                Some(&component.cpp_name),
                format!("`{}` is not an ActorComponent", component.cpp_name),
            ));
        };
        if component.abstract_class {
            return Err(Diagnostic::new(
                "abstract-component",
                Some(&component.cpp_name),
                format!(
                    "`{}` is abstract and cannot be attached",
                    component.cpp_name
                ),
            ));
        }
        if !contract.owners.is_empty() && !contract.owners.contains(&owner.domain) {
            return Err(Diagnostic::new(
                "owner-domain",
                Some(&component.cpp_name),
                format!(
                    "`{}` may only be owned by {} actors; `{}` is {}",
                    component.cpp_name,
                    contract
                        .owners
                        .iter()
                        .map(|d| d.label())
                        .collect::<Vec<_>>()
                        .join("|"),
                    owner.cpp_name,
                    owner.domain.label()
                ),
            ));
        }
        Ok(())
    }
    /// Whole-actor validation: root uniqueness and domain, requires/excludes,
    /// cardinality and target capabilities.
    pub fn validate_component_set(
        &self,
        owner_class: &str,
        components: &[ComponentSpec],
    ) -> Result<(), Vec<Diagnostic>> {
        let Some(owner) = self.class(owner_class) else {
            return Err(vec![Diagnostic::new(
                "unknown-class",
                None,
                format!("`{owner_class}` is not a reflected class"),
            )]);
        };
        let mut diagnostics = Vec::new();
        let mut resolved = Vec::new();
        for spec in components {
            match self.validate_component(&owner.id, &spec.class) {
                Ok(()) => resolved.push((spec, self.class(&spec.class).expect("validated"))),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
        let roots = resolved
            .iter()
            .filter(|(spec, _)| spec.root)
            .collect::<Vec<_>>();
        if roots.len() > 1 {
            diagnostics.push(Diagnostic::new(
                "duplicate-root",
                Some(&owner.cpp_name),
                format!(
                    "`{}` declares {} root components; an actor has exactly one",
                    owner.cpp_name,
                    roots.len()
                ),
            ));
        }
        for (_, class) in &roots {
            let contract = class.component.as_ref().expect("component");
            if !contract.can_root {
                diagnostics.push(Diagnostic::new(
                    "root-not-rootable",
                    Some(&class.cpp_name),
                    format!("`{}` cannot be an actor root", class.cpp_name),
                ));
            }
            if owner.domain.spatial() && class.domain != owner.domain {
                diagnostics.push(Diagnostic::new(
                    "root-domain",
                    Some(&class.cpp_name),
                    format!(
                        "the root of a {} actor must be a {} component; `{}` is {}",
                        owner.domain.label(),
                        owner.domain.label(),
                        class.cpp_name,
                        class.domain.label()
                    ),
                ));
            }
        }
        if owner.domain.spatial() && roots.is_empty() {
            diagnostics.push(Diagnostic::new(
                "missing-root",
                Some(&owner.cpp_name),
                format!(
                    "a {} actor requires exactly one {} root component",
                    owner.domain.label(),
                    owner.domain.label()
                ),
            ));
        }
        // The default target is the one this repository cooks; a caller that knows
        // its target calls `validate_capabilities` with it instead.
        let target = TargetCapabilities::psx();
        for (_, class) in &resolved {
            let contract = class.component.as_ref().expect("component");
            for required in &contract.requires {
                if !resolved
                    .iter()
                    .any(|(_, other)| self.is_a(&other.id, required))
                {
                    diagnostics.push(Diagnostic::new(
                        "missing-requirement",
                        Some(&class.cpp_name),
                        format!(
                            "`{}` requires `{required}` on the same actor",
                            class.cpp_name
                        ),
                    ));
                }
            }
            for excluded in &contract.excludes {
                if resolved
                    .iter()
                    .any(|(_, other)| other.id != class.id && self.is_a(&other.id, excluded))
                {
                    diagnostics.push(Diagnostic::new(
                        "excluded-component",
                        Some(&class.cpp_name),
                        format!("`{}` cannot coexist with `{excluded}`", class.cpp_name),
                    ));
                }
            }
            diagnostics.extend(missing_capabilities(class, &target));
        }
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, class) in &resolved {
            *counts.entry(class.id.as_str()).or_default() += 1;
        }
        for (_, class) in &resolved {
            let contract = class.component.as_ref().expect("component");
            if contract.cardinality == Cardinality::Single && counts[class.id.as_str()] > 1 {
                diagnostics.push(Diagnostic::new(
                    "duplicate-component",
                    Some(&class.cpp_name),
                    format!(
                        "`{}` is Cardinality=Single; the actor declares {} of them",
                        class.cpp_name,
                        counts[class.id.as_str()]
                    ),
                ));
                counts.insert(class.id.as_str(), 1); // one diagnostic per class
            }
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
    /// Build diagnostics for every reflected component whose `Capability=` tokens
    /// the target does not provide. Returned in class order so a build log is
    /// stable; empty means the whole class table can be cooked for that target.
    pub fn validate_capabilities(&self, target: &TargetCapabilities) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for class in self.classes.values() {
            diagnostics.extend(missing_capabilities(class, target));
        }
        diagnostics.sort_by(|a, b| {
            (a.class.as_deref(), &a.message).cmp(&(b.class.as_deref(), &b.message))
        });
        diagnostics
    }

    /// A timeline's `TimelineRequires=` token checked against an actor's component
    /// set. This is the actor-side half of the restriction the timeline adapters
    /// already enforce over legacy entity members; it neither widens nor replaces
    /// them. Requirements with no reserved actor component (`PaletteAnimator`,
    /// `ParticleEmitter`) are not checked here and keep their legacy enforcement.
    pub fn validate_timeline_requirement(
        &self,
        owner_class: &str,
        components: &[ComponentSpec],
        requirement: schema::TimelineComponentRequirement,
    ) -> Result<(), Diagnostic> {
        let Some(required) = timeline_requirement_component(requirement) else {
            return Ok(());
        };
        // An unreflected reserved class cannot be looked up; do not invent a failure.
        if self.class(required).is_none() {
            return Ok(());
        }
        if components
            .iter()
            .any(|spec| self.is_a(&spec.class, required))
        {
            return Ok(());
        }
        let owner = self
            .class(owner_class)
            .map_or(owner_class.to_string(), |c| c.cpp_name.clone());
        Err(Diagnostic::new(
            "missing-timeline-component",
            Some(&owner),
            format!(
                "a timeline track targeting {requirement:?} requires `{required}` on `{owner}`"
            ),
        ))
    }

    /// Actor classes an author may place in a map.
    pub fn placeable(&self) -> impl Iterator<Item = &ClassModel> {
        self.classes
            .values()
            .filter(|c| c.family == ClassFamily::Actor && c.placement.placeable && c.instantiable())
    }
    /// Parents accepted by a scene Blueprint: `SceneScriptActor` and its subclasses.
    pub fn scene_script_parents(&self) -> impl Iterator<Item = &ClassModel> {
        self.classes
            .values()
            .filter(|c| self.is_a(&c.id, SCENE_SCRIPT_ACTOR_ID))
    }
}

/// Depth-first resolution with cycle detection. `Pending` marks the classes on
/// the current path; meeting one again is a cycle.
fn resolve(
    registry: &blueprint::Registry,
    id: &str,
    model: &mut Model,
    state: &mut BTreeMap<String, Resolution>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    match state.get(id) {
        Some(Resolution::Done) => return true,
        Some(Resolution::Failed) => return false,
        Some(Resolution::Pending) => {
            let name = registry
                .classes
                .get(id)
                .map(|c| c.cpp_name.clone())
                .unwrap_or_else(|| id.to_owned());
            diagnostics.push(Diagnostic::new(
                "cyclic-parent",
                Some(&name),
                format!("`{name}` is its own ancestor"),
            ));
            state.insert(id.into(), Resolution::Failed);
            return false;
        }
        None => {}
    }
    let Some(class) = registry.classes.get(id) else {
        return false;
    };
    state.insert(id.into(), Resolution::Pending);
    let resolved = resolve_class(registry, class, model, state, diagnostics);
    match resolved {
        Some(resolved) => {
            state.insert(id.into(), Resolution::Done);
            model.classes.insert(id.into(), resolved);
            true
        }
        None => {
            state.insert(id.into(), Resolution::Failed);
            false
        }
    }
}

/// One diagnostic per `Capability=` token the target does not provide. A class
/// with no component contract declares none, so it never produces one.
fn missing_capabilities(class: &ClassModel, target: &TargetCapabilities) -> Vec<Diagnostic> {
    let Some(contract) = class.component.as_ref() else {
        return Vec::new();
    };
    contract
        .capabilities
        .iter()
        .filter(|capability| !target.provides(capability))
        .map(|capability| {
            Diagnostic::new(
                "unknown-capability",
                Some(&class.cpp_name),
                format!(
                    "`{}` requires the target capability `{capability}`, which this build does not provide",
                    class.cpp_name
                ),
            )
        })
        .collect()
}

fn resolve_class(
    registry: &blueprint::Registry,
    class: &schema::Class,
    model: &mut Model,
    state: &mut BTreeMap<String, Resolution>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ClassModel> {
    let name = class.cpp_name.as_str();
    let parent = match &class.parent {
        Some(parent_id) => {
            if !resolve(registry, parent_id, model, state, diagnostics) {
                if !registry.classes.contains_key(parent_id) {
                    diagnostics.push(Diagnostic::new(
                        "unknown-parent",
                        Some(name),
                        format!("`{name}` derives from the unknown class `{parent_id}`"),
                    ));
                }
                return None;
            }
            Some(model.classes.get(parent_id).expect("resolved").clone())
        }
        None => None,
    };
    // Generated Blueprint C++ is an output of the cook; native code never sees it.
    if let Some(parent) = &parent
        && class.provider == schema::native_provider()
        && parent.provider == script_backend::blueprint_provider()
    {
        diagnostics.push(Diagnostic::new(
            "native-from-blueprint",
            Some(name),
            format!(
                "native class `{name}` cannot derive from generated Blueprint C++ (`{}`)",
                parent.cpp_name
            ),
        ));
        return None;
    }
    let inherited_family = parent.as_ref().map(|p| p.family);
    let family = match (class.family, inherited_family) {
        // `Object` is the universal root: the Actor/Component/World/Level family
        // roots declare themselves directly under it. Any other change is an error.
        (Some(declared), Some(inherited))
            if declared != inherited && inherited != ClassFamily::Object =>
        {
            diagnostics.push(Diagnostic::new(
                "family-mismatch",
                Some(name),
                format!(
                    "`{name}` declares Family={} but inherits {} from its parent",
                    declared.label(),
                    inherited.label()
                ),
            ));
            return None;
        }
        (Some(declared), _) => declared,
        (None, Some(inherited)) => inherited,
        // Reflected value types (for example EffectLayer) have no runtime identity.
        // They remain queryable metadata, but only the Object ancestry is pooled.
        (None, None) => ClassFamily::Object,
    };
    let inherited_domain = parent.as_ref().map(|p| p.domain).unwrap_or(Domain::None);
    let domain = match class.domain {
        Some(declared) if inherited_domain != Domain::None && declared != inherited_domain => {
            diagnostics.push(Diagnostic::new(
                "domain-mismatch",
                Some(name),
                format!(
                    "`{name}` declares Domain={} but inherits {} from its parent",
                    declared.label(),
                    inherited_domain.label()
                ),
            ));
            return None;
        }
        Some(declared) => declared,
        None => inherited_domain,
    };
    let inherited_placement = parent.as_ref().map(|p| p.placement).unwrap_or_default();
    let placement = Placement {
        placeable: class.placement.placeable || inherited_placement.placeable,
        spawnable: class.placement.spawnable || inherited_placement.spawnable,
        scene_managed: class.placement.scene_managed || inherited_placement.scene_managed,
    };
    // SceneManaged wins: the level loader owns these instances.
    let placement = if placement.scene_managed {
        Placement {
            placeable: false,
            spawnable: false,
            scene_managed: true,
        }
    } else {
        placement
    };
    if family != ClassFamily::Actor
        && (placement.placeable || placement.spawnable || placement.scene_managed)
    {
        diagnostics.push(Diagnostic::new(
            "placement-on-non-actor",
            Some(name),
            format!(
                "`{name}` is a {} class; Placeable/Spawnable/SceneManaged apply to Actors",
                family.label()
            ),
        ));
        return None;
    }
    let component = if family == ClassFamily::Component {
        let inherited = parent.as_ref().and_then(|p| p.component.clone());
        Some(merge_component(
            name,
            inherited,
            class.component.as_ref(),
            diagnostics,
        )?)
    } else {
        if class.component.is_some() {
            diagnostics.push(Diagnostic::new(
                "component-contract-on-non-component",
                Some(name),
                format!(
                    "`{name}` is a {} class; Root/Owners/Requires/Excludes/Cardinality/Capability apply to ActorComponents",
                    family.label()
                ),
            ));
            return None;
        }
        None
    };
    let mut ancestry = parent
        .as_ref()
        .map(|p| p.ancestry.clone())
        .unwrap_or_default();
    ancestry.push(class.id.clone());
    // Default components accumulate down the chain; a field name declared again
    // replaces the inherited entry so a subclass can retarget its own defaults.
    let mut default_components = parent
        .as_ref()
        .map(|p| p.default_components.clone())
        .unwrap_or_default();
    for declared in &class.default_components {
        match default_components
            .iter_mut()
            .find(|existing| existing.field == declared.field)
        {
            Some(existing) => *existing = declared.clone(),
            None => default_components.push(declared.clone()),
        }
    }
    Some(ClassModel {
        id: class.id.clone(),
        cpp_name: class.cpp_name.clone(),
        parent: class.parent.clone(),
        family,
        domain,
        placement,
        component,
        abstract_class: class.abstract_class || class.explicit_abstract,
        blueprintable: class.blueprintable,
        final_class: class.final_class,
        default_components,
        provider: class.provider.clone(),
        backend: class.backend.clone(),
        ancestry,
    })
}

/// Merges a declared contract onto the inherited one.
///
/// `owners` may only narrow (an empty declared set inherits); `requires`,
/// `excludes` and `capabilities` accumulate; `can_root` and `Cardinality`
/// widen only, so a subclass never silently loses `Cardinality=Multiple`.
fn merge_component(
    name: &str,
    inherited: Option<ComponentContract>,
    declared: Option<&ComponentContract>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ComponentContract> {
    let mut merged = inherited.unwrap_or_default();
    let Some(declared) = declared else {
        return Some(merged);
    };
    if !declared.owners.is_empty() {
        if !merged.owners.is_empty() && !declared.owners.is_subset(&merged.owners) {
            diagnostics.push(Diagnostic::new(
                "owners-widened",
                Some(name),
                format!(
                    "`{name}` declares Owners={} but its parent allows only {}; a descendant may only narrow the set",
                    labels(&declared.owners),
                    labels(&merged.owners)
                ),
            ));
            return None;
        }
        merged.owners = declared.owners.clone();
    }
    for required in &declared.requires {
        if !merged.requires.contains(required) {
            merged.requires.push(required.clone());
        }
    }
    for excluded in &declared.excludes {
        if !merged.excludes.contains(excluded) {
            merged.excludes.push(excluded.clone());
        }
    }
    merged.capabilities.extend(declared.capabilities.clone());
    merged.can_root |= declared.can_root;
    if declared.cardinality == Cardinality::Multiple {
        merged.cardinality = Cardinality::Multiple;
    }
    Some(merged)
}

fn labels(domains: &BTreeSet<Domain>) -> String {
    domains
        .iter()
        .map(|d| d.label())
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflection_schema::Location;

    /// Hand-built declarations: the tests exercise the resolution rules, not the
    /// Clang extractor (libclang needs the MIPS include paths, absent on hosts).
    struct Decl(schema::Class);
    impl Decl {
        fn new(id: &str, name: &str, parent: Option<&str>) -> Self {
            Self(schema::Class {
                id: id.into(),
                provider: schema::native_provider(),
                backend: schema::native_backend(),
                cpp_name: name.into(),
                parent: parent.map(str::to_owned),
                abstract_class: false,
                final_class: false,
                blueprintable: false,
                timeline_component: None,
                family: None,
                domain: None,
                placement: Placement::default(),
                component: None,
                default_components: vec![],
                explicit_abstract: false,
                properties: vec![],
                functions: vec![],
                source: Location {
                    file: "object_model.hpp".into(),
                    line: 1,
                    column: 1,
                },
            })
        }
        fn family(mut self, family: ClassFamily) -> Self {
            self.0.family = Some(family);
            self
        }
        fn domain(mut self, domain: Domain) -> Self {
            self.0.domain = Some(domain);
            self
        }
        fn abstract_base(mut self) -> Self {
            self.0.explicit_abstract = true;
            self
        }
        fn blueprintable(mut self) -> Self {
            self.0.blueprintable = true;
            self
        }
        fn sealed(mut self) -> Self {
            self.0.final_class = true;
            self
        }
        fn placeable(mut self) -> Self {
            self.0.placement.placeable = true;
            self.0.placement.spawnable = true;
            self
        }
        fn scene_managed(mut self) -> Self {
            self.0.placement.scene_managed = true;
            self
        }
        fn component(mut self, contract: ComponentContract) -> Self {
            self.0.component = Some(contract);
            self
        }
        fn provider(mut self, provider: Extension) -> Self {
            self.0.provider = provider;
            self
        }
        fn build(self) -> schema::Class {
            self.0
        }
    }
    fn contract(owners: &[Domain], can_root: bool) -> ComponentContract {
        ComponentContract {
            owners: owners.iter().copied().collect(),
            can_root,
            ..Default::default()
        }
    }
    fn registry(classes: Vec<schema::Class>) -> blueprint::Registry {
        let mut registry = blueprint::Registry::new();
        for class in classes {
            registry.classes.insert(class.id.clone(), class);
        }
        registry
    }
    fn blueprint_author() -> Extension {
        script_backend::blueprint_provider()
    }
    /// The native bases of design.md section 2, as the extractor would emit them.
    fn native_classes() -> Vec<schema::Class> {
        let all = [Domain::World3D, Domain::World2D, Domain::UI];
        vec![
            Decl::new(OBJECT_ID, "epok::Object", None)
                .family(ClassFamily::Object)
                .abstract_base()
                .build(),
            Decl::new(ACTOR_ID, "epok::Actor", Some(OBJECT_ID))
                .family(ClassFamily::Actor)
                .domain(Domain::None)
                .abstract_base()
                .blueprintable()
                .build(),
            Decl::new(ACTOR3D_ID, "epok::Actor3D", Some(ACTOR_ID))
                .domain(Domain::World3D)
                .blueprintable()
                .placeable()
                .build(),
            Decl::new(ACTOR2D_ID, "epok::Actor2D", Some(ACTOR_ID))
                .domain(Domain::World2D)
                .blueprintable()
                .placeable()
                .build(),
            Decl::new(UI_ACTOR_ID, "epok::UIActor", Some(ACTOR_ID))
                .domain(Domain::UI)
                .blueprintable()
                .placeable()
                .build(),
            Decl::new(
                SCENE_SCRIPT_ACTOR_ID,
                "epok::SceneScriptActor",
                Some(ACTOR_ID),
            )
            .blueprintable()
            .scene_managed()
            .build(),
            Decl::new(ACTOR_COMPONENT_ID, "epok::ActorComponent", Some(OBJECT_ID))
                .family(ClassFamily::Component)
                .domain(Domain::None)
                .abstract_base()
                .blueprintable()
                .build(),
            Decl::new(
                SCENE_COMPONENT3D_ID,
                "epok::SceneComponent3D",
                Some(ACTOR_COMPONENT_ID),
            )
            .domain(Domain::World3D)
            .blueprintable()
            .component(contract(&[Domain::World3D], true))
            .build(),
            Decl::new(
                SCENE_COMPONENT2D_ID,
                "epok::SceneComponent2D",
                Some(ACTOR_COMPONENT_ID),
            )
            .domain(Domain::World2D)
            .blueprintable()
            .component(contract(&[Domain::World2D], true))
            .build(),
            Decl::new(
                UI_COMPONENT_ID,
                "epok::UIComponent",
                Some(ACTOR_COMPONENT_ID),
            )
            .domain(Domain::UI)
            .abstract_base()
            .blueprintable()
            .component(contract(&[Domain::UI], false))
            .build(),
            Decl::new(
                RECT_TRANSFORM_COMPONENT_ID,
                "epok::RectTransformComponent",
                Some(UI_COMPONENT_ID),
            )
            .domain(Domain::UI)
            .blueprintable()
            .component(contract(&[Domain::UI], true))
            .build(),
            Decl::new(
                AUDIO_COMPONENT_ID,
                "epok::AudioComponent",
                Some(ACTOR_COMPONENT_ID),
            )
            .domain(Domain::None)
            .blueprintable()
            .component(ComponentContract {
                owners: all.into_iter().collect(),
                cardinality: Cardinality::Multiple,
                capabilities: ["audio".to_owned()].into_iter().collect(),
                ..Default::default()
            })
            .build(),
            Decl::new(LEVEL_ID, "epok::Level", Some(OBJECT_ID))
                .family(ClassFamily::Level)
                .abstract_base()
                .build(),
            Decl::new(WORLD_ID, "epok::World", Some(OBJECT_ID))
                .family(ClassFamily::World)
                .abstract_base()
                .build(),
        ]
    }
    fn native_model() -> Model {
        Model::from_registry(&registry(native_classes())).expect("native bases resolve")
    }
    fn spec(class: &str, root: bool) -> ComponentSpec {
        ComponentSpec {
            id: uuid::Uuid::new_v4(),
            class: class.into(),
            root,
        }
    }
    fn codes(diagnostics: &[Diagnostic]) -> Vec<&'static str> {
        diagnostics.iter().map(|d| d.code).collect()
    }

    /// Splits `EPOK_CLASS(` ... `)` for one declaration. The header contains no
    /// nested parentheses inside the macro, so a scan to the first `)` is exact.
    fn header_declarations(header: &str) -> Vec<(String, Vec<String>)> {
        let mut declarations = Vec::new();
        for (index, _) in header.match_indices("class EPOK_CLASS(") {
            let rest = &header[index + "class EPOK_CLASS(".len()..];
            let stop = rest.find(')').expect("closing parenthesis");
            let options = rest[..stop]
                .split(',')
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let name = rest[stop + 1..]
                .trim_start()
                .split([' ', ':', '{', '\n'])
                .next()
                .expect("class name")
                .to_owned();
            declarations.push((name, options));
        }
        declarations
    }

    #[test]
    fn native_identities_match_the_runtime_header() {
        let header = include_str!("../runtime/object_model.hpp");
        let expected: BTreeMap<&str, &str> = [
            ("Object", OBJECT_ID),
            ("Actor", ACTOR_ID),
            ("Actor3D", ACTOR3D_ID),
            ("Actor2D", ACTOR2D_ID),
            ("UIActor", UI_ACTOR_ID),
            ("SceneScriptActor", SCENE_SCRIPT_ACTOR_ID),
            ("ActorComponent", ACTOR_COMPONENT_ID),
            ("SceneComponent3D", SCENE_COMPONENT3D_ID),
            ("SceneComponent2D", SCENE_COMPONENT2D_ID),
            ("UIComponent", UI_COMPONENT_ID),
            ("RectTransformComponent", RECT_TRANSFORM_COMPONENT_ID),
            ("AudioComponent", AUDIO_COMPONENT_ID),
            ("Mesh3DComponent", MESH3D_COMPONENT_ID),
            ("Sprite3DComponent", SPRITE3D_COMPONENT_ID),
            ("Camera3DComponent", CAMERA3D_COMPONENT_ID),
            ("Light3DComponent", LIGHT3D_COMPONENT_ID),
            ("Collider3DComponent", COLLIDER3D_COMPONENT_ID),
            ("CanvasComponent", CANVAS_COMPONENT_ID),
            ("ImageComponent", IMAGE_COMPONENT_ID),
            ("TextComponent", TEXT_COMPONENT_ID),
            ("ProgressBarComponent", PROGRESS_BAR_COMPONENT_ID),
            (
                "ParticleEmitterComponent",
                crate::actor_components::PARTICLES,
            ),
            ("TimelineComponent", crate::actor_components::TIMELINE),
            ("ParticleEffectComponent", crate::actor_components::EFFECT),
            ("PaletteAnimatorComponent", crate::actor_components::PALETTE),
            ("BlobShadowComponent", crate::actor_components::SHADOW),
            ("Level", LEVEL_ID),
            ("World", WORLD_ID),
        ]
        .into_iter()
        .collect();
        let declarations = header_declarations(header);
        assert_eq!(declarations.len(), expected.len(), "{declarations:?}");
        for (name, options) in &declarations {
            let id = options
                .iter()
                .find_map(|option| option.strip_prefix("Id="))
                .map(|value| value.trim_matches('"'))
                .unwrap_or_else(|| panic!("{name} declares no Id"));
            assert_eq!(
                expected.get(name.as_str()),
                Some(&id),
                "{name} identity drifted from design.md section 2"
            );
            assert!(uuid::Uuid::parse_str(id).is_ok(), "{name}: {id}");
        }
        // Every reserved P3/P8 identity is a distinct UUID.
        let reserved = [
            SPRITE2D_COMPONENT_ID,
            CAMERA2D_COMPONENT_ID,
            COLLIDER2D_COMPONENT_ID,
        ];
        let unique = reserved
            .iter()
            .chain(expected.values())
            .collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), reserved.len() + expected.len());
        assert!(reserved.iter().all(|id| uuid::Uuid::parse_str(id).is_ok()));
    }

    #[test]
    fn native_bases_resolve_to_the_documented_contract() {
        let model = native_model();
        let actor3d = model.class("epok::Actor3D").expect("Actor3D");
        assert_eq!(actor3d.family, ClassFamily::Actor);
        assert_eq!(actor3d.domain, Domain::World3D);
        assert!(actor3d.placement.placeable && actor3d.placement.spawnable);
        assert_eq!(
            actor3d.ancestry,
            vec![
                OBJECT_ID.to_owned(),
                ACTOR_ID.to_owned(),
                ACTOR3D_ID.to_owned()
            ]
        );
        // Abstract, non-instantiable bases stay in the ancestry table.
        assert!(model.is_a("epok::Actor3D", "epok::Object"));
        assert!(!model.class("epok::Actor").expect("Actor").instantiable());
        assert!(!model.is_a("epok::Actor3D", "epok::Actor2D"));

        let scene_script = model
            .class(SCENE_SCRIPT_ACTOR_ID)
            .expect("SceneScriptActor");
        assert_eq!(scene_script.domain, Domain::None);
        assert!(scene_script.placement.scene_managed);
        assert!(!scene_script.placement.placeable && !scene_script.placement.spawnable);

        let audio = model.class("epok::AudioComponent").expect("AudioComponent");
        let audio_contract = audio.component.as_ref().expect("contract");
        assert_eq!(audio_contract.cardinality, Cardinality::Multiple);
        assert!(!audio_contract.can_root);
        // A component inherits its family from ActorComponent without redeclaring it.
        assert_eq!(audio.family, ClassFamily::Component);
    }

    #[test]
    fn cpp_and_blueprint_chains_resolve_and_blueprint_never_parents_native() {
        let mut classes = native_classes();
        // C++ -> C++
        classes.push(
            Decl::new("cpp:Pawn", "Pawn", Some(ACTOR3D_ID))
                .blueprintable()
                .build(),
        );
        // C++ -> BP -> BP
        classes.push(
            Decl::new("bp:Hero", "BP_Hero", Some("cpp:Pawn"))
                .provider(blueprint_author())
                .blueprintable()
                .build(),
        );
        classes.push(
            Decl::new("bp:Knight", "BP_Knight", Some("bp:Hero"))
                .provider(blueprint_author())
                .blueprintable()
                .build(),
        );
        let model = Model::from_registry(&registry(classes.clone())).expect("chain resolves");
        let knight = model.class("BP_Knight").expect("BP_Knight");
        assert_eq!(knight.family, ClassFamily::Actor);
        assert_eq!(knight.domain, Domain::World3D);
        assert!(knight.placement.placeable, "placement is inherited");
        assert!(model.is_a("BP_Knight", "epok::Actor3D"));
        assert_eq!(knight.ancestry.len(), 6);

        // A native class deriving from generated Blueprint C++ is rejected.
        classes.push(Decl::new("cpp:Wrong", "Wrong", Some("bp:Hero")).build());
        let diagnostics = Model::from_registry(&registry(classes)).unwrap_err();
        assert_eq!(codes(&diagnostics), ["native-from-blueprint"]);
        assert_eq!(diagnostics[0].class.as_deref(), Some("Wrong"));
    }

    #[test]
    fn missing_and_cyclic_parents_are_reported() {
        let diagnostics = Model::from_registry(&registry(vec![
            Decl::new("cpp:A", "A", Some("cpp:Ghost")).build(),
        ]))
        .unwrap_err();
        assert_eq!(codes(&diagnostics), ["unknown-parent"]);
        assert!(diagnostics[0].message.contains("cpp:Ghost"));

        let cyclic = Model::from_registry(&registry(vec![
            Decl::new("cpp:A", "A", Some("cpp:B")).build(),
            Decl::new("cpp:B", "B", Some("cpp:A")).build(),
        ]))
        .unwrap_err();
        assert!(cyclic.iter().any(|d| d.code == "cyclic-parent"));
    }

    #[test]
    fn redeclaring_a_different_family_or_domain_is_a_diagnostic() {
        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Odd", "Odd", Some(ACTOR3D_ID))
                .family(ClassFamily::Component)
                .build(),
        );
        let diagnostics = Model::from_registry(&registry(classes)).unwrap_err();
        assert_eq!(codes(&diagnostics), ["family-mismatch"]);

        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Flat", "Flat", Some(ACTOR3D_ID))
                .domain(Domain::UI)
                .build(),
        );
        let diagnostics = Model::from_registry(&registry(classes)).unwrap_err();
        assert_eq!(codes(&diagnostics), ["domain-mismatch"]);

        // Declaring the same value again is legal, as is refining Domain=None.
        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Same", "Same", Some(ACTOR3D_ID))
                .family(ClassFamily::Actor)
                .domain(Domain::World3D)
                .build(),
        );
        assert!(Model::from_registry(&registry(classes)).is_ok());
    }

    #[test]
    fn component_owners_may_only_narrow() {
        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Narrow", "Narrow", Some(AUDIO_COMPONENT_ID))
                .component(contract(&[Domain::UI], false))
                .build(),
        );
        let model = Model::from_registry(&registry(classes)).expect("narrowing is legal");
        let narrow = model.class("Narrow").expect("Narrow");
        let narrowed = narrow.component.as_ref().expect("contract");
        assert_eq!(narrowed.owners, [Domain::UI].into_iter().collect());
        // Inherited cardinality and capabilities survive the narrowing.
        assert_eq!(narrowed.cardinality, Cardinality::Multiple);
        assert!(narrowed.capabilities.contains("audio"));

        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Wide", "Wide", Some(UI_COMPONENT_ID))
                .component(contract(&[Domain::UI, Domain::World3D], false))
                .build(),
        );
        let diagnostics = Model::from_registry(&registry(classes)).unwrap_err();
        assert_eq!(codes(&diagnostics), ["owners-widened"]);
        assert!(diagnostics[0].message.contains("World3D"));
    }

    #[test]
    fn component_contracts_and_placement_stay_inside_their_family() {
        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Bad", "Bad", Some(ACTOR3D_ID))
                .component(contract(&[Domain::World3D], true))
                .build(),
        );
        assert_eq!(
            codes(&Model::from_registry(&registry(classes)).unwrap_err()),
            ["component-contract-on-non-component"]
        );

        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Placed", "Placed", Some(SCENE_COMPONENT3D_ID))
                .placeable()
                .build(),
        );
        assert_eq!(
            codes(&Model::from_registry(&registry(classes)).unwrap_err()),
            ["placement-on-non-actor"]
        );
    }

    #[test]
    fn reparent_checks_family_domain_derivability_and_cycles() {
        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Pawn", "Pawn", Some(ACTOR3D_ID))
                .blueprintable()
                .build(),
        );
        classes.push(
            Decl::new("bp:Hero", "BP_Hero", Some("cpp:Pawn"))
                .provider(blueprint_author())
                .blueprintable()
                .build(),
        );
        classes.push(
            Decl::new("cpp:Sealed", "Sealed", Some(ACTOR3D_ID))
                .blueprintable()
                .sealed()
                .build(),
        );
        classes.push(Decl::new("cpp:Opaque", "Opaque", Some(ACTOR3D_ID)).build());
        let model = Model::from_registry(&registry(classes)).expect("model");

        assert!(model.validate_reparent("BP_Hero", "epok::Actor3D").is_ok());
        // An Actor may not be reparented onto a Component.
        let crossing = model
            .validate_reparent("BP_Hero", "epok::SceneComponent3D")
            .unwrap_err();
        assert!(crossing.iter().any(|d| d.code == "family-crossing"));
        // World3D actor onto a UI actor.
        let domain = model
            .validate_reparent("cpp:Pawn", UI_ACTOR_ID)
            .unwrap_err();
        assert!(domain.iter().any(|d| d.code == "domain-mismatch"));
        // Final and non-blueprintable parents are refused.
        for parent in ["Sealed", "Opaque"] {
            let refused = model.validate_reparent("BP_Hero", parent).unwrap_err();
            assert!(refused.iter().any(|d| d.code == "parent-not-derivable"));
        }
        // A class may not become its own descendant's child.
        let cycle = model.validate_reparent("cpp:Pawn", "BP_Hero").unwrap_err();
        assert!(cycle.iter().any(|d| d.code == "cyclic-parent"));
        assert!(model.validate_reparent("cpp:Pawn", "cpp:Pawn").is_err());
        assert!(model.validate_reparent("cpp:Pawn", "epok::Ghost").is_err());
    }

    #[test]
    fn component_domains_gate_attachment() {
        let model = native_model();
        // The UI component family may not be attached to a 3D actor.
        let refused = model
            .validate_component(ACTOR3D_ID, RECT_TRANSFORM_COMPONENT_ID)
            .unwrap_err();
        assert_eq!(refused.code, "owner-domain");
        assert!(
            model
                .validate_component(UI_ACTOR_ID, RECT_TRANSFORM_COMPONENT_ID)
                .is_ok()
        );
        // A domain-less component with Owners=World3D|World2D|UI fits every actor.
        for owner in [ACTOR3D_ID, ACTOR2D_ID, UI_ACTOR_ID] {
            assert!(
                model.validate_component(owner, AUDIO_COMPONENT_ID).is_ok(),
                "AudioComponent on {owner}"
            );
        }
        // Abstract components and non-actor owners are refused.
        assert_eq!(
            model
                .validate_component(UI_ACTOR_ID, UI_COMPONENT_ID)
                .unwrap_err()
                .code,
            "abstract-component"
        );
        assert_eq!(
            model
                .validate_component(SCENE_COMPONENT3D_ID, AUDIO_COMPONENT_ID)
                .unwrap_err()
                .code,
            "not-an-actor"
        );
        assert_eq!(
            model
                .validate_component(ACTOR3D_ID, ACTOR2D_ID)
                .unwrap_err()
                .code,
            "not-a-component"
        );
    }

    #[test]
    fn a_spatial_actor_needs_exactly_one_matching_root() {
        let model = native_model();
        assert!(
            model
                .validate_component_set(ACTOR3D_ID, &[spec(SCENE_COMPONENT3D_ID, true)])
                .is_ok()
        );
        // No root at all.
        let missing = model
            .validate_component_set(ACTOR3D_ID, &[spec(AUDIO_COMPONENT_ID, false)])
            .unwrap_err();
        assert_eq!(codes(&missing), ["missing-root"]);
        // Two roots.
        let duplicate = model
            .validate_component_set(
                ACTOR3D_ID,
                &[
                    spec(SCENE_COMPONENT3D_ID, true),
                    spec(SCENE_COMPONENT3D_ID, true),
                ],
            )
            .unwrap_err();
        assert!(duplicate.iter().any(|d| d.code == "duplicate-root"));
        // A component that cannot root.
        let unrootable = model
            .validate_component_set(ACTOR3D_ID, &[spec(AUDIO_COMPONENT_ID, true)])
            .unwrap_err();
        assert!(unrootable.iter().any(|d| d.code == "root-not-rootable"));
        // A domain-less actor needs no root.
        assert!(
            model
                .validate_component_set(SCENE_SCRIPT_ACTOR_ID, &[])
                .is_ok()
        );
    }

    #[test]
    fn requires_excludes_cardinality_and_capabilities() {
        let mut classes = native_classes();
        classes.push(
            Decl::new(
                "cpp:Body",
                "epok::BodyComponent",
                Some(SCENE_COMPONENT3D_ID),
            )
            .domain(Domain::World3D)
            .component(ComponentContract {
                requires: vec!["epok::AudioComponent".into()],
                excludes: vec!["epok::ExoticComponent".into()],
                ..Default::default()
            })
            .build(),
        );
        classes.push(
            Decl::new(
                "cpp:Exotic",
                "epok::ExoticComponent",
                Some(ACTOR_COMPONENT_ID),
            )
            .domain(Domain::None)
            .component(ComponentContract {
                // A name no target in this repository provides.
                capabilities: ["shader-graph".to_owned()].into_iter().collect(),
                ..Default::default()
            })
            .build(),
        );
        let model = Model::from_registry(&registry(classes)).expect("model");

        // Requires is satisfied by any subclass of the named class.
        assert!(
            model
                .validate_component_set(
                    ACTOR3D_ID,
                    &[spec("cpp:Body", true), spec(AUDIO_COMPONENT_ID, false)]
                )
                .is_ok()
        );
        let missing = model
            .validate_component_set(ACTOR3D_ID, &[spec("cpp:Body", true)])
            .unwrap_err();
        assert_eq!(codes(&missing), ["missing-requirement"]);
        let excluded = model
            .validate_component_set(
                ACTOR3D_ID,
                &[
                    spec("cpp:Body", true),
                    spec(AUDIO_COMPONENT_ID, false),
                    spec("cpp:Exotic", false),
                ],
            )
            .unwrap_err();
        assert_eq!(
            codes(&excluded),
            ["excluded-component", "unknown-capability"]
        );

        // Cardinality: Single rejects duplicates, Multiple accepts them.
        let single = model
            .validate_component_set(
                ACTOR3D_ID,
                &[
                    spec(SCENE_COMPONENT3D_ID, true),
                    spec("cpp:Body", false),
                    spec("cpp:Body", false),
                    spec(AUDIO_COMPONENT_ID, false),
                ],
            )
            .unwrap_err();
        assert_eq!(codes(&single), ["duplicate-component"]);
        assert!(
            model
                .validate_component_set(
                    ACTOR3D_ID,
                    &[
                        spec(SCENE_COMPONENT3D_ID, true),
                        spec(AUDIO_COMPONENT_ID, false),
                        spec(AUDIO_COMPONENT_ID, false),
                    ],
                )
                .is_ok()
        );
        // An unknown capability names itself so the exporter can map it.
        let capability = model
            .validate_component_set(
                ACTOR3D_ID,
                &[spec(SCENE_COMPONENT3D_ID, true), spec("cpp:Exotic", false)],
            )
            .unwrap_err();
        assert_eq!(codes(&capability), ["unknown-capability"]);
        assert!(capability[0].message.contains("shader-graph"));
    }

    #[test]
    fn target_capabilities_come_from_the_target_not_a_hard_coded_list() {
        let psx = TargetCapabilities::psx();
        for name in [
            "audio",
            "hud",
            "sprites",
            "particles",
            "timelines",
            "physics3d",
        ] {
            assert!(psx.provides(name), "PSX must provide `{name}`");
        }
        // physics2d follows world2d.hpp, which this repository does cook.
        assert!(psx.provides("physics2d"));
        assert!(TargetCapabilities::psx_with_world2d(true).provides("physics2d"));
        assert!(!TargetCapabilities::psx_with_world2d(false).provides("physics2d"));
        assert!(!psx.provides("shader-graph"));
        assert!(!TargetCapabilities::none().provides("audio"));
        assert!(TargetCapabilities::none().with("audio").provides("audio"));
        assert!(!psx.clone().without("audio").provides("audio"));
        assert!(psx.names().count() >= 7);

        // The whole class table validates against the target it will be cooked for.
        let model = native_model();
        assert!(
            model.validate_capabilities(&psx).is_empty(),
            "the native bases cook for PSX"
        );
        // A target without audio reports AudioComponent by name, once.
        let deaf = TargetCapabilities::psx().without("audio");
        let diagnostics = model.validate_capabilities(&deaf);
        assert_eq!(codes(&diagnostics), ["unknown-capability"]);
        assert_eq!(
            diagnostics[0].class.as_deref(),
            Some("epok::AudioComponent")
        );
        assert!(diagnostics[0].message.contains("audio"));
        // A target that provides nothing reports exactly the classes that declare
        // a capability, not every component.
        let bare = model.validate_capabilities(&TargetCapabilities::none());
        assert_eq!(
            bare.iter()
                .filter_map(|d| d.class.as_deref())
                .collect::<Vec<_>>(),
            ["epok::AudioComponent"]
        );
    }

    #[test]
    fn timeline_requirements_map_onto_the_reserved_component_classes() {
        use schema::TimelineComponentRequirement as R;
        // Every mapped requirement names a class reserved in design.md section 2.
        assert_eq!(
            timeline_requirement_component(R::AudioSource),
            Some("epok::AudioComponent")
        );
        assert_eq!(
            timeline_requirement_component(R::Camera),
            Some("epok::Camera3DComponent")
        );
        assert_eq!(
            timeline_requirement_component(R::Light),
            Some("epok::Light3DComponent")
        );
        assert_eq!(
            timeline_requirement_component(R::RectTransform),
            Some("epok::RectTransformComponent")
        );
        assert_eq!(
            timeline_requirement_component(R::Text),
            Some("epok::TextComponent")
        );
        assert_eq!(
            timeline_requirement_component(R::Image),
            Some("epok::ImageComponent")
        );
        assert_eq!(
            timeline_requirement_component(R::ProgressBar),
            Some("epok::ProgressBarComponent")
        );
        // Not yet reserved: these keep their legacy-entity-only enforcement.
        assert_eq!(timeline_requirement_component(R::PaletteAnimator), None);
        assert_eq!(timeline_requirement_component(R::ParticleEmitter), None);
    }

    #[test]
    fn timeline_requirement_diagnoses_an_actor_without_the_component() {
        let model = native_model();
        let root = spec(SCENE_COMPONENT3D_ID, true);
        // The actor carries the component the timeline needs.
        assert!(
            model
                .validate_timeline_requirement(
                    ACTOR3D_ID,
                    &[root.clone(), spec(AUDIO_COMPONENT_ID, false)],
                    schema::TimelineComponentRequirement::AudioSource,
                )
                .is_ok()
        );
        // It does not: one diagnostic naming both the actor and the component.
        let diagnostic = model
            .validate_timeline_requirement(
                ACTOR3D_ID,
                std::slice::from_ref(&root),
                schema::TimelineComponentRequirement::AudioSource,
            )
            .unwrap_err();
        assert_eq!(diagnostic.code, "missing-timeline-component");
        assert_eq!(diagnostic.class.as_deref(), Some("epok::Actor3D"));
        assert!(diagnostic.message.contains("epok::AudioComponent"));
        // A requirement whose component class is not reflected yet never fails:
        // Camera3DComponent is reserved but not declared in the native bases.
        assert!(
            model
                .validate_timeline_requirement(
                    ACTOR3D_ID,
                    std::slice::from_ref(&root),
                    schema::TimelineComponentRequirement::Camera,
                )
                .is_ok()
        );
        // Neither does one with no reserved class at all.
        assert!(
            model
                .validate_timeline_requirement(
                    ACTOR3D_ID,
                    &[root],
                    schema::TimelineComponentRequirement::ParticleEmitter,
                )
                .is_ok()
        );
    }

    #[test]
    fn eligible_parents_placeable_and_scene_script_parents() {
        let mut classes = native_classes();
        classes.push(
            Decl::new("cpp:Pawn", "Pawn", Some(ACTOR3D_ID))
                .blueprintable()
                .build(),
        );
        classes.push(
            Decl::new("cpp:Sealed", "Sealed", Some(ACTOR3D_ID))
                .blueprintable()
                .sealed()
                .build(),
        );
        classes.push(Decl::new("cpp:Opaque", "Opaque", Some(ACTOR3D_ID)).build());
        classes.push(
            Decl::new("cpp:MapScript", "MapScript", Some(SCENE_SCRIPT_ACTOR_ID))
                .blueprintable()
                .build(),
        );
        let model = Model::from_registry(&registry(classes)).expect("model");

        let parents = model
            .eligible_parents(&blueprint_author(), ClassFamily::Actor)
            .into_iter()
            .map(|c| c.cpp_name.as_str())
            .collect::<Vec<_>>();
        assert!(parents.contains(&"Pawn") && parents.contains(&"epok::Actor3D"));
        // Final and non-blueprintable classes are never offered as parents.
        assert!(!parents.contains(&"Sealed") && !parents.contains(&"Opaque"));
        // Families never mix in the picker.
        assert!(!parents.contains(&"epok::AudioComponent"));
        assert!(
            model
                .eligible_parents(&blueprint_author(), ClassFamily::Component)
                .iter()
                .any(|c| c.cpp_name == "epok::AudioComponent")
        );

        let placeable = model
            .placeable()
            .map(|c| c.cpp_name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            placeable,
            [
                "Opaque",
                "Pawn",
                "Sealed",
                "epok::Actor2D",
                "epok::Actor3D",
                "epok::UIActor"
            ]
            .into_iter()
            .collect()
        );
        // SceneScriptActor is created by the loader and is never placeable.
        assert!(!placeable.contains("epok::SceneScriptActor") && !placeable.contains("MapScript"));

        let scene_scripts = model
            .scene_script_parents()
            .map(|c| c.cpp_name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            scene_scripts,
            ["MapScript", "epok::SceneScriptActor"]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn default_components_accumulate_down_the_chain() {
        let mut classes = native_classes();
        let mut pawn = Decl::new("cpp:Pawn", "Pawn", Some(ACTOR3D_ID))
            .blueprintable()
            .build();
        pawn.default_components = vec![DefaultComponent {
            id: "cpp:Pawn:body".into(),
            field: "body".into(),
            class: "epok::SceneComponent3D".into(),
            root: true,
            attach_to: None,
            name: Some("Body".into()),
        }];
        classes.push(pawn);
        let mut hero = Decl::new("bp:Hero", "BP_Hero", Some("cpp:Pawn"))
            .provider(blueprint_author())
            .blueprintable()
            .build();
        hero.default_components = vec![DefaultComponent {
            id: "bp:Hero:voice".into(),
            field: "voice".into(),
            class: "epok::AudioComponent".into(),
            root: false,
            attach_to: Some("body".into()),
            name: None,
        }];
        classes.push(hero);
        let model = Model::from_registry(&registry(classes)).expect("model");
        let fields = model
            .class("BP_Hero")
            .expect("BP_Hero")
            .default_components
            .iter()
            .map(|c| c.field.as_str())
            .collect::<Vec<_>>();
        assert_eq!(fields, ["body", "voice"]);
    }

    #[test]
    fn registry_exposes_the_model() {
        let model = registry(native_classes()).model().expect("model");
        assert_eq!(model.len(), native_classes().len());
        assert!(!model.is_empty());
        assert!(model.class(ACTOR3D_ID).is_some());
    }
}
