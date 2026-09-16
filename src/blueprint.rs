//! Shared registry for authoring. Execution metadata is supplied by its provider.
use crate::{reflection_schema as schema, scripts::Script};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug)]
pub struct Registry {
    pub classes: BTreeMap<String, schema::Class>,
    /// One compiler-facing operation catalog for instance methods and static
    /// function libraries. Frontends never maintain independent signatures.
    pub operations: BTreeMap<String, schema::Operation>,
    pub function_libraries: BTreeMap<String, schema::FunctionLibrary>,
    pub value_types: BTreeMap<String, schema::ValueDeclaration>,
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
                .map(|f| {
                    (
                        f.id.clone(),
                        (
                            f.callable,
                            f.event,
                            f.pure,
                            f.timeline,
                            f.resource_demands.clone(),
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let mut changed = false;
            for f in self
                .classes
                .values_mut()
                .flat_map(|class| class.functions.iter_mut())
            {
                for parent in &f.overrides {
                    if let Some((callable, event, pure, timeline, resource_demands)) =
                        flags.get(parent)
                    {
                        let before = (f.callable, f.event, f.pure, f.timeline);
                        let before_demands = f.resource_demands.clone();
                        f.callable |= *callable;
                        f.event |= *event;
                        f.pure |= *pure;
                        f.timeline = f.timeline.or(*timeline);
                        f.resource_demands.extend(resource_demands.iter().cloned());
                        f.resource_demands.sort();
                        f.resource_demands.dedup();
                        changed |= before != (f.callable, f.event, f.pure, f.timeline)
                            || before_demands != f.resource_demands;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        self.rebuild_operations();
    }
    pub fn new() -> Self {
        Self {
            classes: BTreeMap::new(),
            operations: BTreeMap::new(),
            function_libraries: BTreeMap::new(),
            value_types: BTreeMap::new(),
        }
    }
    fn operation_parameter(
        operation: &str,
        index: usize,
        parameter: &schema::Parameter,
    ) -> schema::OperationParameter {
        schema::OperationParameter {
            id: format!("{operation}:parameter:{index}"),
            name: parameter.name.clone(),
            value_type: parameter.value_type.clone(),
            direction: parameter.direction.clone(),
            default: None,
        }
    }
    fn operation_demands(
        class: &schema::Class,
        declared: impl IntoIterator<Item = String>,
    ) -> Vec<String> {
        let mut demands = class
            .component
            .as_ref()
            .map(|contract| contract.capabilities.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        demands.extend(declared);
        demands.sort();
        demands.dedup();
        demands
    }
    fn instance_operation(class: &schema::Class, function: &schema::Function) -> schema::Operation {
        let outputs = (function.returns != schema::Type::Void)
            .then(|| schema::OperationOutput {
                id: format!("{}:result", function.id),
                name: "result".into(),
                value_type: function.returns.clone(),
            })
            .into_iter()
            .collect();
        schema::Operation {
            version: 1,
            id: function.id.clone(),
            namespace: class.cpp_name.clone(),
            name: function.name.clone(),
            category: class.cpp_name.clone(),
            search_terms: vec![function.name.replace('_', " ")],
            receiver: schema::ReceiverKind::Instance {
                class: class.id.clone(),
            },
            native_target: format!("{}::{}", class.cpp_name, function.name),
            parameters: function
                .parameters
                .iter()
                .enumerate()
                .map(|(index, parameter)| Self::operation_parameter(&function.id, index, parameter))
                .collect(),
            outputs,
            effect: if function.pure {
                schema::OperationEffect::StateRead
            } else {
                schema::OperationEffect::Mutation
            },
            can_destroy_receiver: matches!(function.name.as_str(), "destroy" | "destroy_actor"),
            valid_domains: class.domain.into_iter().collect(),
            component_requirements: class
                .component
                .as_ref()
                .map(|contract| contract.requires.clone())
                .unwrap_or_default(),
            resource_demands: Self::operation_demands(
                class,
                function.resource_demands.iter().cloned(),
            ),
            error_contract:
                "Invalid/stale receivers return the type default and perform no mutation".into(),
            complexity: "O(1) unless the native declaration documents a bounded traversal".into(),
            source: function.source.clone(),
        }
    }
    fn property_operation(
        class: &schema::Class,
        property: &schema::Property,
        write: bool,
    ) -> schema::Operation {
        let suffix = if write { "set" } else { "get" };
        let id = format!("{}:{suffix}", property.id);
        schema::Operation {
            version: 1,
            id: id.clone(),
            namespace: class.cpp_name.clone(),
            name: format!("{suffix}_{}", property.name),
            category: class.cpp_name.clone(),
            search_terms: vec![property.name.replace('_', " ")],
            receiver: schema::ReceiverKind::Instance {
                class: class.id.clone(),
            },
            native_target: format!("property:{suffix}:{}", property.name),
            parameters: if write {
                vec![schema::OperationParameter {
                    id: format!("{id}:parameter:0"),
                    name: "value".into(),
                    value_type: property.value_type.clone(),
                    direction: schema::Direction::Value,
                    default: None,
                }]
            } else {
                vec![]
            },
            outputs: if write {
                vec![]
            } else {
                vec![schema::OperationOutput {
                    id: format!("{id}:result"),
                    name: "result".into(),
                    value_type: property.value_type.clone(),
                }]
            },
            effect: if write {
                schema::OperationEffect::Mutation
            } else {
                schema::OperationEffect::StateRead
            },
            can_destroy_receiver: false,
            valid_domains: class.domain.into_iter().collect(),
            component_requirements: class
                .component
                .as_ref()
                .map(|value| value.requires.clone())
                .unwrap_or_default(),
            resource_demands: Self::operation_demands(class, std::iter::empty()),
            error_contract:
                "Invalid/stale receivers return the type default and perform no mutation".into(),
            complexity: "O(1) direct reflected field access".into(),
            source: property.source.clone(),
        }
    }
    fn rebuild_operations(&mut self) {
        self.operations.clear();
        for library in self.function_libraries.values() {
            for operation in &library.operations {
                self.operations
                    .insert(operation.id.clone(), operation.clone());
            }
        }
        for class in self.classes.values() {
            for function in &class.functions {
                if function.callable {
                    let operation = Self::instance_operation(class, function);
                    self.operations.insert(operation.id.clone(), operation);
                }
            }
            for property in &class.properties {
                let getter = Self::property_operation(class, property, false);
                self.operations.insert(getter.id.clone(), getter);
                if property.editable {
                    let setter = Self::property_operation(class, property, true);
                    self.operations.insert(setter.id.clone(), setter);
                }
            }
        }
    }
    pub fn operation(&self, id: &str) -> Option<&schema::Operation> {
        self.operations.get(id)
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
    pub fn lua_parents(&self) -> impl Iterator<Item = &schema::Class> {
        self.classes.values().filter(|class| {
            crate::script_backend::can_derive(&crate::script_backend::lua_provider(), class)
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
    let manifest = crate::reflection::discover(root)?;
    for class in manifest.classes {
        registry.classes.insert(class.id.clone(), class);
    }
    for library in manifest.function_libraries {
        registry
            .function_libraries
            .insert(library.id.clone(), library);
    }
    for value in manifest.value_types {
        registry.value_types.insert(value.id.clone(), value);
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
