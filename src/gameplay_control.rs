//! Routed input bindings retained for controller comparisons and contract tests.
#![cfg(test)]
use crate::reflection_schema::Type;

pub const HELD: &str = "26561bbe-17e7-5273-9fb8-3b9470cdfdba";
pub const PRESSED: &str = "7161d176-d7c9-5216-92cf-e6f9a89b0360";
#[cfg(test)]
pub const AXIS: &str = "e0e243c1-bdd8-5930-95c6-c1e0ab449bbc";

pub fn button_type() -> Type {
    Type::Enum {
        cpp_name: "epok::Button".into(),
        variants: [
            "Select", "L3", "R3", "Start", "Up", "Right", "Down", "Left", "L2", "R2", "L1", "R1",
            "Triangle", "Circle", "Cross", "Square",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name.into(), index as i64))
        .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_control_contract_is_reflected_with_pinned_ids_and_effects() {
        let root = crate::workspace::tests::temp("gameplay-control-registry");
        crate::workspace::create(&root, "Control Contract", crate::workspace::Template::Basic)
            .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let identities: serde_json::Value = serde_json::from_str(include_str!(
            "../knowledge/initiatives/gameplay-helpers/identities.json"
        ))
        .unwrap();
        let library = registry
            .function_libraries
            .get(
                identities["identities"]["GameplayInputLibrary"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(library.operations.len(), 23);
        for op in &library.operations {
            assert_eq!(op.id, identities["identities"][&op.name].as_str().unwrap());
            assert!(op.resource_demands.iter().any(|c| c == "gameplay-control"));
            if op.name.contains("take_") {
                assert_eq!(
                    op.effect,
                    crate::reflection_schema::OperationEffect::EventConsumption
                );
            }
        }
        for (name, count) in [("InputCapture", 2), ("GameplayActionSample", 5)] {
            let value = registry
                .value_types
                .get(identities["identities"][name].as_str().unwrap())
                .unwrap();
            assert_eq!(value.value_type.members().len(), count);
        }
        assert_eq!(
            registry.operations[HELD].parameters[0].value_type,
            button_type()
        );
        assert!(registry.operations.contains_key(PRESSED));
        assert!(registry.operations.contains_key(AXIS));
        std::fs::remove_dir_all(root).unwrap();
    }
}
