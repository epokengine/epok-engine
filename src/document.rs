//! Text documents owned by Epok. External protocols and binary package metadata use JSON.
//!
//! Use the same plain data model as our versioned schemas, expressed as YAML.
//! The intermediate value avoids language-specific YAML enum tags and keeps map
//! ordering deterministic. JSON syntax is accepted on read for legacy fixtures.
use serde::{Serialize, de::DeserializeOwned};

pub fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let value: serde_yaml_ng::Value =
        serde_yaml_ng::from_slice(bytes).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    serde_json::from_value(value).map_err(|e| e.to_string())
}

#[allow(dead_code)] // Also compiled into the standalone header extractor.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    from_slice(text.as_bytes())
}

#[allow(dead_code)]
pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, String> {
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    serde_yaml_ng::to_string(&value)
        .map(String::into_bytes)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn yaml_preserves_schema_values_and_is_deterministic() {
        let data = json!({"name":"yes", "id":"001", "script":{"default":null},
            "unicode":"Cámara", "float":1.25, "boolean":true,
            "nested":[{"kind":"Fixed","value":42}], "expression":"a: b # c"});
        let bytes = to_vec(&data).unwrap();
        assert!(!bytes.starts_with(b"{"));
        assert_eq!(from_slice::<serde_json::Value>(&bytes).unwrap(), data);
        assert_eq!(
            to_vec(&from_slice::<serde_json::Value>(&bytes).unwrap()).unwrap(),
            bytes
        );
    }

    #[test]
    fn rejects_duplicate_fields_and_multiple_documents() {
        assert!(from_str::<serde_json::Value>("a: 1\na: 2\n").is_err());
        assert!(from_str::<serde_json::Value>("a: 1\n---\na: 2\n").is_err());
    }
}
