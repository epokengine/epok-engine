//! Shared registry for authoring. Execution metadata is supplied by its provider.
use crate::{reflection_schema as schema, scripts::Script};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug)]
pub struct Registry {
    pub classes: BTreeMap<String, schema::Class>,
}
impl Registry {
    /// Native overrides inherit exposure from their original declaration even
    /// when C++ does not repeat the annotation on the implementation.
    pub fn normalize_functions(&mut self) {
        for _ in 0..self.classes.len() {
            let flags = self
                .classes
                .values()
                .flat_map(|class| class.functions.iter())
                .map(|f| (f.id.clone(), (f.callable, f.event, f.pure, f.timeline)))
                .collect::<BTreeMap<_, _>>();
            let mut changed = false;
            for f in self
                .classes
                .values_mut()
                .flat_map(|class| class.functions.iter_mut())
            {
                for parent in &f.overrides {
                    if let Some(&(callable, event, pure, timeline)) = flags.get(parent) {
                        let before = (f.callable, f.event, f.pure, f.timeline);
                        f.callable |= callable;
                        f.event |= event;
                        f.pure |= pure;
                        f.timeline = f.timeline.or(timeline);
                        changed |= before != (f.callable, f.event, f.pure, f.timeline);
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
    pub fn new() -> Self {
        Self {
            classes: BTreeMap::new(),
        }
    }
    pub fn eligible_parents(&self) -> impl Iterator<Item = &schema::Class> {
        self.classes
            .values()
            .filter(|c| crate::script_backend::can_derive(&schema::native_provider(), c))
    }
    pub fn blueprint_parents(&self) -> impl Iterator<Item = &schema::Class> {
        self.classes.values().filter(|class| {
            crate::script_backend::can_derive(&crate::script_backend::blueprint_provider(), class)
        })
    }
    pub fn named(&self, name: &str) -> Option<&schema::Class> {
        self.classes.values().find(|c| c.cpp_name == name)
    }
    pub fn bound(&self, binding: &crate::scene::ClassDefaults) -> Option<&schema::Class> {
        if let Some(id) = &binding.class_id {
            self.classes.get(id)
        } else {
            self.named(&binding.name)
        }
    }
    pub fn upgrade_binding(&self, binding: &mut crate::scene::ClassDefaults) {
        if crate::script_backend::validate_binding(binding).is_err() {
            return;
        }
        if let Some(class) = self.bound(binding) {
            if binding.class_id.is_none() {
                binding.class_id = Some(class.id.clone());
                binding.provider = class.provider.clone();
                binding.backend = class.backend.clone();
            }
            for property in self.properties(&class.cpp_name) {
                if binding.properties.contains_key(&property.name) {
                    binding
                        .member_ids
                        .entry(property.name.clone())
                        .or_insert_with(|| property.id.clone());
                }
            }
        }
    }
    pub fn ancestry(&self, name: &str) -> Vec<&schema::Class> {
        let mut chain = vec![];
        let mut current = self.named(name);
        while let Some(class) = current {
            if chain.iter().any(|c: &&schema::Class| c.id == class.id) {
                break;
            }
            chain.push(class);
            current = class.parent.as_ref().and_then(|id| self.classes.get(id));
        }
        chain.reverse();
        chain
    }
    /// Resolved Object/Actor/Component model for this registry (schema 8).
    /// The model is derived, never cached here: callers that need it repeatedly
    /// build it once per registry change.
    #[allow(dead_code)] // P1 publishes the model; its editor consumers arrive in P2+.
    pub fn model(
        &self,
    ) -> Result<crate::object_model::Model, Vec<crate::object_model::Diagnostic>> {
        crate::object_model::Model::from_registry(self)
    }
    pub fn properties(&self, name: &str) -> Vec<&schema::Property> {
        // Legacy catalog entries already contain flattened inherited fields.
        // Resolve by name once; semantic classes reject shadowing during discovery.
        let mut properties = BTreeMap::new();
        for class in self.ancestry(name) {
            for property in &class.properties {
                properties.insert(&property.name, property);
            }
        }
        properties.into_values().collect()
    }
}
/// Obtain the actual root signatures even in a project with no user C++ class.
pub fn native_registry(root: &Path, scripts: &[Script]) -> Result<Registry, String> {
    let mut registry = registry_from_catalog(root, scripts);
    // Native script chains omit non-behaviour SDK targets such as EffectLayer.
    // Merge the same authoritative Clang manifest, including those declarations.
    for class in crate::reflection::discover(root)?.classes {
        registry.classes.insert(class.id.clone(), class);
    }
    registry.normalize_functions();
    Ok(registry)
}
/// Legacy sidecars remain a compatibility provider. They have no fabricated function
/// signatures; annotated native classes retain the declarations extracted by Clang.
pub fn registry_from_catalog(root: &Path, scripts: &[Script]) -> Registry {
    let mut registry = Registry::new();
    let _ = root;
    for script in scripts {
        for class in &script.classes {
            registry.classes.insert(class.id.clone(), class.clone());
        }
    }
    registry.normalize_functions();
    registry
}
