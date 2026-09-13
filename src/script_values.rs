//! Typed property validation, target assignments, and Inspector controls.
use crate::reflection_schema::Type;
use serde_json::{Value, json};

pub fn default_value(ty: &Type) -> Value {
    if matches!(ty, Type::Record { .. }) && !ty.members().is_empty() {
        return Value::Object(ty.members().into_iter().map(|field| {
            let value = if matches!(ty, Type::Record { cpp_name, .. } if cpp_name == "epok::Transform") && field.name == "scale" {
                json!([1, 1, 1])
            } else { default_value(&field.value_type) };
            (field.name, value)
        }).collect());
    }
    match ty {
        Type::Bool => json!(false),
        Type::Vector { length } => json!(vec![0.; *length]),
        Type::Enum { variants, .. } => json!(variants.values().next().copied().unwrap_or_default()),
        Type::EntityRef { .. }
        | Type::ObjectRef { .. }
        | Type::ActorRef { .. }
        | Type::ComponentRef { .. }
        | Type::EffectLayerRef { .. }
        | Type::AssetRef { .. }
        | Type::ClassRef { .. } => Value::Null,
        Type::SequenceHandle | Type::EffectHandle => Value::Null,
        _ => json!(0),
    }
}

pub fn valid(value: &Value, ty: &Type) -> bool {
    if matches!(ty, Type::Record { .. }) && !ty.members().is_empty() {
        let fields = ty.members();
        return value.as_object().is_some_and(|object| {
            object.len() == fields.len()
                && fields.iter().all(|field| {
                    object
                        .get(&field.name)
                        .is_some_and(|v| valid(v, &field.value_type))
                })
        });
    }
    match ty {
        Type::SequenceHandle | Type::EffectHandle => value.is_null(),
        Type::Bool => value.is_boolean(),
        Type::Int32 => value.as_i64().is_some_and(|v| i32::try_from(v).is_ok()),
        Type::UInt32 => value.as_u64().is_some_and(|v| u32::try_from(v).is_ok()),
        Type::Fixed => value.as_f64().is_some_and(|v| {
            v.is_finite()
                && (-524288.0..524288.0).contains(&v)
                && (v * 4096.0).round() <= f64::from(i32::MAX)
        }),
        Type::Vector { length } => value
            .as_array()
            .is_some_and(|v| v.len() == *length && v.iter().all(|v| valid(v, &Type::Fixed))),
        Type::Enum { variants, .. } => value
            .as_i64()
            .is_some_and(|v| variants.values().any(|item| *item == v)),
        Type::EntityRef { .. }
        | Type::ObjectRef { .. }
        | Type::ActorRef { .. }
        | Type::ComponentRef { .. }
        | Type::EffectLayerRef { .. }
        | Type::AssetRef { .. } => {
            value.is_null()
                || value
                    .as_str()
                    .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok_and(|id| !id.is_nil()))
        }
        Type::ClassRef { .. } => {
            value.is_null()
                || value
                    .as_str()
                    .is_some_and(|id| !id.is_empty() && id.len() <= 512)
        }
        _ => false,
    }
}

pub fn assignment(target: &str, value: &Value, ty: &Type) -> Result<String, String> {
    if !valid(value, ty) {
        return Err(format!(
            "{target}: value {value} does not match {ty:?}; preserved override requires migration"
        ));
    }
    let expression = match ty {
        Type::Record { .. } if !ty.members().is_empty() => {
            return ty
                .members()
                .iter()
                .map(|field| {
                    assignment(
                        &format!("{target}.{}", field.name),
                        &value[&field.name],
                        &field.value_type,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.join(""));
        }
        Type::Bool | Type::Int32 => value.to_string(),
        Type::UInt32 => format!("{value}u"),
        Type::Fixed => format!(
            "Fixed({}, Fixed::RAW)",
            (value.as_f64().unwrap() * 4096.0).round() as i32
        ),
        Type::Enum { cpp_name, .. } => format!("static_cast<{cpp_name}>({value})"),
        Type::Vector { length } => {
            return (0..*length)
                .map(|i| assignment(&format!("{target}[{i}]"), &value[i], &Type::Fixed))
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.join(""));
        }
        Type::EntityRef { .. } if value.is_null() => "epok::EntityHandle{}".into(),
        // Typed object references are compact generational ids. Persisted content only
        // ever carries the null identity; live ids are resolved at spawn time.
        Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
            if value.is_null() =>
        {
            "epok::ObjectId{}".into()
        }
        Type::SequenceHandle => "epok::timeline::Handle{}".into(),
        Type::EffectHandle => "epok::effects::Handle{}".into(),
        Type::ClassRef { .. } | Type::AssetRef { .. } => value
            .as_str()
            .map(|id| format!("UINT64_C({})", crate::blueprint_refs::compact_id(id)))
            .unwrap_or_else(|| "UINT64_C(0)".into()),
        _ => return Err("No native value adapter for this type".into()),
    };
    Ok(format!("{target} = {expression};\n"))
}

pub fn inspector(ui: &imgui::Ui, id: &str, value: &mut Value, ty: &Type) -> bool {
    if !valid(value, ty) {
        ui.text_wrapped("Stored value has an incompatible type. Reset explicitly or migrate it; the value is preserved.");
        return false;
    }
    match ty {
        Type::Record { .. } if !ty.members().is_empty() => {
            let _id = ui.push_id(id);
            ui.text(id);
            let mut changed = false;
            for field in ty.members() {
                changed |= inspector(ui, &field.name, &mut value[&field.name], &field.value_type);
            }
            return changed;
        }
        Type::Bool => {
            let mut v = value.as_bool().unwrap();
            if ui.checkbox(id, &mut v) {
                *value = json!(v);
                return true;
            }
        }
        Type::Int32 => {
            let mut v = value.as_i64().unwrap() as i32;
            if crate::gui::Drag::new(id).speed(1.).build(ui, &mut v) {
                *value = json!(v);
                return true;
            }
        }
        Type::UInt32 => {
            let mut v = value.as_u64().unwrap() as u32;
            if crate::gui::Drag::new(id).speed(1.).build(ui, &mut v) {
                *value = json!(v);
                return true;
            }
        }
        Type::Fixed => {
            let mut v = value.as_f64().unwrap() as f32;
            if crate::gui::Drag::new(id)
                .speed(0.25)
                .range(-10000., 10000.)
                .build(ui, &mut v)
            {
                *value = json!(v);
                return true;
            }
        }
        Type::Vector { length } => {
            let mut values = value
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect::<Vec<_>>();
            if crate::gui::Drag::new(id)
                .speed(0.25)
                .range(-10000., 10000.)
                .build_array(ui, &mut values[..*length])
            {
                *value = json!(values);
                return true;
            }
        }
        Type::Enum { variants, .. } => {
            let preview = variants
                .iter()
                .find(|(_, v)| Some(**v) == value.as_i64())
                .map(|(name, _)| name.as_str())
                .unwrap_or("Unknown");
            if let Some(_combo) = ui.begin_combo(id, preview) {
                for (name, v) in variants {
                    if ui
                        .selectable_config(name)
                        .selected(Some(*v) == value.as_i64())
                        .build()
                    {
                        *value = json!(v);
                        return true;
                    }
                }
            }
        }
        Type::EntityRef { .. } | Type::AssetRef { .. } | Type::ClassRef { .. } => {
            let mut id_text = value.as_str().unwrap_or_default().to_owned();
            if ui
                .input_text(id, &mut id_text)
                .hint("None or persistent ID")
                .build()
            {
                let candidate = if id_text.trim().is_empty() {
                    Value::Null
                } else {
                    json!(id_text.trim())
                };
                if valid(&candidate, ty) {
                    *value = candidate;
                    return true;
                }
            }
        }
        _ => ui.text_disabled("No editable adapter for this type"),
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn values_are_not_coerced_and_fixed_uses_target_q12() {
        assert!(!valid(&json!(1), &Type::Bool));
        assert!(!valid(&json!(1.5), &Type::Int32));
        assert!(!valid(&json!(-1), &Type::UInt32));
        assert!(valid(&json!(-524288.0), &Type::Fixed));
        assert!(valid(&json!(f64::from(i32::MAX) / 4096.0), &Type::Fixed));
        assert!(!valid(&json!(524287.9999), &Type::Fixed));
        assert!(!valid(&json!(524288.0), &Type::Fixed));
        assert_eq!(
            assignment("v", &json!(1.25), &Type::Fixed).unwrap(),
            "v = Fixed(5120, Fixed::RAW);\n"
        );
        assert_eq!(
            assignment("v", &json!([1.0, 2.0]), &Type::Vector { length: 2 }).unwrap(),
            "v[0] = Fixed(4096, Fixed::RAW);\nv[1] = Fixed(8192, Fixed::RAW);\n"
        );
    }
}
