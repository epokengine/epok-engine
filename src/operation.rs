//! Shared resolved gameplay-operation calls.
//!
//! Blueprint and Lua keep their existing control-flow IRs, but operation
//! identity, receiver semantics, effects and resource demands are resolved once
//! into this representation. Emitters only adapt their expression payload.
use crate::{reflection_schema as schema, script_ir::Span};

#[derive(Clone, Debug)]
pub enum Receiver<T> {
    Instance { value: T, value_type: schema::Type },
    Service { library: String },
    Value { value: T, value_type: schema::Type },
}

#[derive(Clone, Debug)]
pub struct Argument<T> {
    pub parameter_id: String,
    pub value: T,
    pub value_type: schema::Type,
}

#[derive(Clone, Debug)]
pub struct ResolvedCall<T> {
    pub operation_id: String,
    pub receiver: Receiver<T>,
    pub arguments: Vec<Argument<T>>,
    pub outputs: Vec<schema::OperationOutput>,
    pub effect: schema::OperationEffect,
    pub resource_demands: Vec<String>,
    pub span: Span,
}

impl<T> ResolvedCall<T> {
    pub fn from_operation(
        operation: &schema::Operation,
        receiver: Receiver<T>,
        arguments: Vec<Argument<T>>,
        span: Span,
    ) -> Result<Self, String> {
        if operation.parameters.len() != arguments.len() {
            return Err(format!(
                "{} expects {} arguments, got {}",
                operation.name,
                operation.parameters.len(),
                arguments.len()
            ));
        }
        for (parameter, argument) in operation.parameters.iter().zip(&arguments) {
            if parameter.id != argument.parameter_id || parameter.value_type != argument.value_type
            {
                return Err(format!(
                    "{} argument {} does not match the resolved operation contract",
                    operation.name, parameter.name
                ));
            }
        }
        Ok(Self {
            operation_id: operation.id.clone(),
            receiver,
            arguments,
            outputs: operation.outputs.clone(),
            effect: operation.effect,
            resource_demands: operation.resource_demands.clone(),
            span,
        })
    }

    pub fn reentrant(&self) -> bool {
        matches!(
            self.effect,
            schema::OperationEffect::Mutation
                | schema::OperationEffect::EventConsumption
                | schema::OperationEffect::AsyncRequest
                | schema::OperationEffect::ResourceAcquisition
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn operation() -> schema::Operation {
        schema::Operation {
            version: 1,
            id: "operation-id".into(),
            namespace: "epok::Input".into(),
            name: "held".into(),
            category: "Input".into(),
            search_terms: vec!["button".into()],
            receiver: schema::ReceiverKind::Service {
                library: "epok::Input".into(),
            },
            native_target: "epok::bp::api::held".into(),
            parameters: vec![schema::OperationParameter {
                id: "operation-id:parameter:0".into(),
                name: "button".into(),
                value_type: schema::Type::UInt32,
                direction: schema::Direction::Value,
                default: None,
            }],
            outputs: vec![schema::OperationOutput {
                id: "operation-id:result".into(),
                name: "result".into(),
                value_type: schema::Type::Bool,
            }],
            effect: schema::OperationEffect::StateRead,
            can_destroy_receiver: false,
            valid_domains: BTreeSet::new(),
            component_requirements: vec![],
            resource_demands: vec![],
            error_contract: "invalid ports return false".into(),
            complexity: "O(1)".into(),
            source: schema::Location {
                file: "gameplay_api.hpp".into(),
                line: 1,
                column: 1,
            },
        }
    }

    #[test]
    fn resolved_call_rejects_signature_drift() {
        let operation = operation();
        let receiver = Receiver::Service {
            library: "epok::Input".into(),
        };
        let wrong = vec![Argument {
            parameter_id: "operation-id:parameter:0".into(),
            value: "button".to_string(),
            value_type: schema::Type::Int32,
        }];
        assert!(
            ResolvedCall::from_operation(&operation, receiver, wrong, Span::default()).is_err()
        );
    }

    #[test]
    fn gameplay_api_parity_preserves_effect_outputs_and_resource_demands() {
        let mut operation = operation();
        operation.effect = schema::OperationEffect::AsyncRequest;
        operation.resource_demands = vec!["skeletal-vertex-query".into()];
        let argument = Argument {
            parameter_id: "operation-id:parameter:0".into(),
            value: "button".to_string(),
            value_type: schema::Type::UInt32,
        };
        let call = ResolvedCall::from_operation(
            &operation,
            Receiver::Service {
                library: "epok::Input".into(),
            },
            vec![argument],
            Span::default(),
        )
        .unwrap();
        assert!(call.reentrant());
        assert_eq!(call.outputs, operation.outputs);
        assert_eq!(call.resource_demands, ["skeletal-vertex-query"]);
    }
}
