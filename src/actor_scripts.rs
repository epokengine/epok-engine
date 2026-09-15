//! Transactional attachment of native and Blueprint ActorComponents.
use crate::{
    actor_document::{ActorInstance, ClassReference, ComponentInstance},
    blueprint::Registry,
    object_model::{ClassModel, Model},
    reflection_schema::{Class, ClassFamily},
    scene::Scene,
};
use uuid::Uuid;

/// The Project browser can author any class; Add Component keeps its original
/// owner and only offers parents whose descendants can belong to that actor.
#[derive(Clone, Copy, Default)]
pub enum CreationContext {
    #[default]
    Project,
    Component(Uuid),
}

impl CreationContext {
    pub fn actor_index(self, scene: &Scene, selected: Option<usize>) -> Option<usize> {
        match self {
            Self::Project => selected,
            Self::Component(actor) => scene.actor_index(actor),
        }
    }

    pub fn allows_parent(self, scene: &Scene, model: Option<&Model>, parent: &Class) -> bool {
        let Self::Component(actor) = self else {
            return true;
        };
        let Some(model) = model else { return false };
        let Some(parent) = model.class(&parent.id) else {
            return false;
        };
        let Some(owner) = scene
            .actor_index(actor)
            .and_then(|index| scene.actors[index].class.resolve(model))
        else {
            return false;
        };
        // Abstract parents are valid: the newly authored child supplies behavior.
        parent.family == ClassFamily::Component
            && parent.component.as_ref().is_some_and(|contract| {
                contract.owners.is_empty() || contract.owners.contains(&owner.domain)
            })
    }
}

pub fn owner(scene: &Scene, index: usize) -> Option<&ActorInstance> {
    scene.actors.get(index)
}
pub fn validate_target(
    scene: &Scene,
    index: usize,
    class: &ClassModel,
    model: &Model,
) -> Result<(), String> {
    let actor = scene.actors.get(index).ok_or("Select an Actor first.")?;
    if class.family != ClassFamily::Component {
        return Err("Add Component accepts ActorComponent classes. Create an Actor from the Hierarchy to instantiate an Actor class.".into());
    }
    let owner = actor
        .class
        .resolve(model)
        .ok_or("The Actor class is unresolved.")?;
    model
        .validate_component(&owner.id, &class.id)
        .map_err(|error| error.message)
}
pub fn validate_parent(
    scene: &Scene,
    index: usize,
    parent: &str,
    registry: &Registry,
) -> Result<(), String> {
    validate_parent_with_owner(scene, index, parent, registry, None)
}
pub fn validate_parent_with_owner(
    scene: &Scene,
    index: usize,
    parent: &str,
    registry: &Registry,
    owner: Option<crate::reflection_schema::Domain>,
) -> Result<(), String> {
    let parent = registry
        .classes
        .get(parent)
        .ok_or("Select an ActorComponent parent.")?;
    let mut candidates = registry.clone();
    let mut child = parent.clone();
    child.id = Uuid::new_v4().to_string();
    child.cpp_name = "BlueprintAttachmentPreview".into();
    child.parent = Some(parent.id.clone());
    child.provider = crate::script_backend::blueprint_provider();
    child.family = None;
    child.explicit_abstract = false;
    child.abstract_class = false;
    child.default_components.clear();
    if let Some(owner) = owner {
        let model = registry.model().map_err(|errors| {
            errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let mut contract = model
            .class(&parent.id)
            .and_then(|c| c.component.clone())
            .ok_or("Select an ActorComponent parent.")?;
        contract.owners = [owner].into_iter().collect();
        child.component = Some(contract);
    }
    candidates.classes.insert(child.id.clone(), child.clone());
    let model = candidates.model().map_err(|errors| {
        errors
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    validate_target(
        scene,
        index,
        model
            .class(&child.id)
            .ok_or("The component class is unresolved.")?,
        &model,
    )?;
    let mut candidate = scene.actors[index].clone();
    candidate.components.push(ComponentInstance::new(
        Uuid::new_v4(),
        ClassReference::new(&child.cpp_name, &child.id),
        "ComponentPreview",
    ));
    crate::mcp_tools::validate_actor_components(&model, &candidate)
}
pub fn assign(
    scene: &Scene,
    index: usize,
    class: &Class,
    registry: &Registry,
) -> Result<Scene, String> {
    if class.abstract_class || class.explicit_abstract {
        return Err("Implement the abstract functions before adding this component.".into());
    }
    let model = registry.model().map_err(|errors| {
        errors
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let resolved = model
        .class(&class.id)
        .ok_or("The component class is unresolved.")?;
    validate_target(scene, index, resolved, &model)?;
    let mut candidate = scene.clone();
    let actor = &mut candidate.actors[index];
    let name = crate::actor_document::unique_component_name(
        actor,
        crate::actor_document::short_class_name(&class.cpp_name),
    );
    actor.components.push(ComponentInstance::new(
        Uuid::new_v4(),
        ClassReference::new(&class.cpp_name, &class.id),
        &name,
    ));
    crate::mcp_tools::validate_actor_components(&model, actor)?;
    actor.refresh_components();
    candidate.validate_with_model(Some(&model))?;
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{actor_document::tests as fixture, object_model as om};

    #[test]
    fn component_creation_parents_follow_family_and_inherited_owner_domains() {
        let mut registry = fixture::registry();
        let controller = fixture::class("controller", "Controller", Some(om::ACTOR_COMPONENT_ID));
        let mut blueprint = fixture::class("bp-controller", "BP_Controller", Some("controller"));
        blueprint.provider = crate::script_backend::blueprint_provider();
        let actor = fixture::class("actor-child", "ControllerActor", Some(om::ACTOR3D_ID));
        let ui = fixture::class("ui-child", "WidgetLogic", Some(om::UI_COMPONENT_ID));
        for class in [controller, blueprint, actor, ui] {
            registry.classes.insert(class.id.clone(), class);
        }
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        for (class, spatial) in [
            (om::ACTOR3D_ID, om::SCENE_COMPONENT3D_ID),
            (om::ACTOR2D_ID, om::SCENE_COMPONENT2D_ID),
            (om::UI_ACTOR_ID, om::UI_COMPONENT_ID),
        ] {
            let actor =
                ActorInstance::new(Uuid::new_v4(), ClassReference::new(class, class), "Owner");
            let context = CreationContext::Component(actor.id);
            scene.actors.push(actor);
            let accepts =
                |id: &str| context.allows_parent(&scene, Some(&model), &registry.classes[id]);
            for allowed in [
                om::ACTOR_COMPONENT_ID,
                om::AUDIO_COMPONENT_ID,
                "controller",
                "bp-controller",
                spatial,
            ] {
                assert!(accepts(allowed), "{class} should offer {allowed}");
            }
            for rejected in [
                om::OBJECT_ID,
                om::ACTOR_ID,
                om::ACTOR3D_ID,
                om::ACTOR2D_ID,
                om::UI_ACTOR_ID,
                om::SCENE_SCRIPT_ACTOR_ID,
                "actor-child",
            ] {
                assert!(!accepts(rejected), "{class} must not offer {rejected}");
            }
            assert_eq!(accepts("ui-child"), class == om::UI_ACTOR_ID);
            assert_eq!(accepts(om::SCENE_COMPONENT3D_ID), class == om::ACTOR3D_ID);
            assert_eq!(accepts(om::SCENE_COMPONENT2D_ID), class == om::ACTOR2D_ID);
            assert_eq!(
                context.actor_index(&scene, None),
                Some(scene.actors.len() - 1)
            );
        }
        for class in registry.classes.values() {
            assert!(CreationContext::Project.allows_parent(&scene, Some(&model), class));
        }
        let missing = CreationContext::Component(Uuid::new_v4());
        assert!(!missing.allows_parent(
            &scene,
            Some(&model),
            &registry.classes[om::ACTOR_COMPONENT_ID]
        ));
        assert!(missing.actor_index(&scene, Some(0)).is_none());
    }
}
