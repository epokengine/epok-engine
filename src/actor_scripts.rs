//! Transactional attachment of native and Blueprint ActorComponents.
use crate::{
    actor_document::{ActorInstance, ClassReference, ComponentInstance},
    blueprint::Registry,
    object_model::{ClassModel, Model},
    reflection_schema::{Class, ClassFamily},
    scene::Scene,
};
use uuid::Uuid;

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
    )
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
