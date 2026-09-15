//! Typed C++ adapters for the shared cooked TimelineAsset tables.
use crate::{blueprint::Registry, reflection_schema::Type, timeline_compile::Compiled};

/// Reuse Blueprint's compact identities and reject collisions before emitting
/// tables. This is cook-time validation, not another runtime identity registry.
pub fn validate_ids<'a>(
    assets: impl IntoIterator<Item = &'a Compiled>,
    registry: &Registry,
) -> Result<(), String> {
    let mut identities = std::collections::BTreeMap::new();
    let mut check = |source: String| -> Result<(), String> {
        let compact = crate::blueprint_refs::compact_id(&source);
        if compact == 0
            || identities
                .insert(compact, source.clone())
                .is_some_and(|previous| previous != source)
        {
            return Err(format!("Timeline runtime identity collision for {source}"));
        }
        Ok(())
    };
    for class in registry.classes.values() {
        check(class.id.clone())?;
    }
    for asset in assets {
        check(asset.asset.to_string())?;
        for track in &asset.tracks {
            check(track.property.clone())?;
        }
        for (_, marker) in &asset.markers {
            check(marker.to_string())?;
        }
        for event in &asset.events {
            for argument in &event.arguments {
                if let Some(resource) = argument.resource {
                    check(resource.to_string())?;
                }
            }
        }
    }
    Ok(())
}

fn from_raw(ty: &Type, raw: &str) -> Result<String, String> {
    Ok(match ty {
        Type::Fixed => format!("epok::Fixed({raw},epok::Fixed::RAW)"),
        Type::Bool => format!("({raw}!=0)"),
        Type::UInt32 => format!("uint32_t({raw})"),
        Type::Int32 => raw.into(),
        Type::Enum { cpp_name, .. } => format!("static_cast<{cpp_name}>({raw})"),
        _ => return Err("No cooked scalar adapter for the reflected type".into()),
    })
}
fn raw_field(ty: &Type, field: &str) -> String {
    if matches!(ty, Type::Fixed) {
        format!("{field}.raw()")
    } else {
        format!("int32_t({field})")
    }
}
pub fn header(compiled: &Compiled, registry: &Registry) -> Result<String, String> {
    validate_ids([compiled], registry)?;
    let mut out = compiled.tables();
    out += "#include \"timeline_runtime.hpp\"\n#include \"blueprint_spawn.hpp\"\n";
    let stem = compiled.asset.simple();
    out += &format!("namespace epok::timeline::cooked::asset_{stem} {{\n");
    let mut target_entries = vec![];
    for (i, slot) in compiled.slots.iter().enumerate() {
        let check = match &slot.target {
            Type::ObjectRef { class: Some(class) }
            | Type::ActorRef { class: Some(class) }
            | Type::ComponentRef { class: Some(class) } => {
                let mut check = format!(
                    "!target.internal&&target.get()&&epok::bp::is_a(target,UINT64_C({}))",
                    crate::blueprint_refs::compact_id(class)
                );
                let class = registry
                    .classes
                    .get(class)
                    .ok_or("Missing compiled target class")?;
                for component in registry
                    .ancestry(&class.cpp_name)
                    .iter()
                    .filter_map(|class| class.timeline_component)
                {
                    check += &format!(
                        "&&target.data()&&target.data()->{}",
                        component.runtime_member()
                    );
                }
                check
            }
            Type::EffectLayerRef { class } if class == crate::particle_effect::LAYER_CLASS_ID => {
                "target.internal&&target.effect_layer()".into()
            }
            _ => return Err("Unsupported cooked target type".into()),
        };
        out += &format!("inline bool accepts_{i}(BoundTarget target){{return {check};}}\n");
        target_entries.push(format!("{{{},accepts_{i}}}", slot.required));
    }
    let slot_index = |id| {
        compiled
            .slots
            .iter()
            .position(|s| s.id == id)
            .ok_or("Missing compiled binding slot")
    };
    let mut property_entries = vec![];
    let receiver = |slot: usize, class: &str| {
        if matches!(compiled.slots[slot].target, Type::EffectLayerRef { .. }) {
            format!(
                "if(!accepts_{slot}(target)||!target.active())return false;auto* object=target.effect_layer();"
            )
        } else {
            format!(
                "if(!accepts_{slot}(target)||!target.active())return false;auto* base=epok::bp::object(target);if(!base)return false;auto* object=static_cast<{class}*>(base);"
            )
        }
    };
    for (i, track) in compiled.tracks.iter().enumerate() {
        let slot = slot_index(track.slot)?;
        let receiver = receiver(slot, &track.class);
        out += &format!("inline bool read_{i}(BoundTarget target,Value& value){{{receiver}");
        let sync = |read: bool| {
            if matches!(compiled.slots[slot].target, Type::EffectLayerRef { .. }) {
                String::new()
            } else {
                format!(
                    "object->timeline_sync(UINT64_C({}),{read});",
                    crate::blueprint_refs::compact_id(&track.property)
                )
            }
        };
        out += &sync(true);
        for lane in 0..track.channels.len() {
            let (ty, field) = if matches!(track.value_type, Type::Vector { .. }) {
                (&Type::Fixed, format!("object->{}[{lane}]", track.field))
            } else {
                (&track.value_type, format!("object->{}", track.field))
            };
            out += &format!("value.lanes[{lane}]={};", raw_field(ty, &field));
        }
        out += "return true;}\n";
        out += &format!("inline bool write_{i}(BoundTarget target,const Value& value){{{receiver}");
        for lane in 0..track.channels.len() {
            let (ty, field) = if matches!(track.value_type, Type::Vector { .. }) {
                (&Type::Fixed, format!("object->{}[{lane}]", track.field))
            } else {
                (&track.value_type, format!("object->{}", track.field))
            };
            out += &format!(
                "{field}={};",
                from_raw(ty, &format!("value.lanes[{lane}]"))?
            );
        }
        out += &sync(false);
        out += "return true;}\n";
        property_entries.push(format!(
            "{{UINT64_C({}),{slot},{},{},{},curves_{},read_{i},write_{i},{}}}",
            crate::blueprint_refs::compact_id(&track.property),
            track.channels.len(),
            track.blend == crate::timeline::Blend::Additive,
            track.restore == crate::timeline::Restore::RestoreInitial,
            track.id.simple(),
            track.range.map_or_else(
                || "0,INT32_MAX,0,1,1".into(),
                |r| format!(
                    "{},{},{},{},{}",
                    r.start, r.end, r.offset, r.numerator, r.denominator
                )
            )
        ));
    }
    let mut event_entries = vec![];
    for (i, event) in compiled.events.iter().enumerate() {
        let slot = slot_index(event.slot)?;
        let function = registry
            .ancestry(&event.class)
            .into_iter()
            .rev()
            .flat_map(|c| &c.functions)
            .find(|f| f.id == event.function || f.overrides.contains(&event.function))
            .ok_or("Missing timeline function during runtime generation")?;
        let native_arrays = registry
            .classes
            .values()
            .find(|c| c.source.file == function.source.file)
            .is_none_or(|c| c.provider.id != "blueprint");
        out += &format!(
            "inline bool event_{i}(BoundTarget target,const BoundTarget* bindings,const Argument* arguments){{{}",
            receiver(slot, &event.class)
        );
        let mut args = vec![];
        for (n, arg) in event.arguments.iter().enumerate() {
            let value = if arg.slot.is_some() {
                format!("bindings[arguments[{n}].slot]")
            } else if matches!(arg.value_type, Type::AssetRef { .. }) {
                format!("arguments[{n}].resource")
            } else if let Type::Vector { length } = &arg.value_type {
                let name = format!("argument_{n}");
                out += &if native_arrays {
                    format!("epok::Fixed {name}[{length}];")
                } else {
                    format!("epok::bp::Vector<{length}> {name};")
                };
                for lane in 0..*length {
                    out += &format!(
                        "{name}[{lane}]=epok::Fixed(arguments[{n}].lanes[{lane}],epok::Fixed::RAW);"
                    );
                }
                name
            } else {
                from_raw(&arg.value_type, &format!("arguments[{n}].lanes[0]"))?
            };
            args.push(value);
        }
        let call = format!("object->{}({});return true;", event.method, args.join(","));
        // Reuse Blueprint's existing dynamic-instance quarantine around a
        // native call: destroying the receiver cannot recycle its live stack.
        if matches!(compiled.slots[slot].target, Type::EffectLayerRef { .. }) {
            out += &format!("{call}}}\n");
        } else {
            out += &format!(
                "if(!epok::active_object_registry)return false;epok::ObjectDispatchScope scope(*epok::active_object_registry);{call}}}\n"
            );
        }
        event_entries.push(format!(
            "{{{slot},{},{},{},event_{i}}}",
            event.arguments.len(),
            event.call == crate::reflection_schema::TimelineCall::IdempotentAction,
            if event.arguments.is_empty() {
                "nullptr".into()
            } else {
                format!("arguments_{}", event.key.simple())
            }
        ));
    }
    for (ty, name, entries) in [
        ("Target", "targets", target_entries),
        ("Property", "properties", property_entries),
        ("Event", "events", event_entries),
    ] {
        if !entries.is_empty() {
            out += &format!("inline const {ty} {name}[]={{{}}};\n", entries.join(","));
        }
    }
    if !compiled.markers.is_empty() {
        out += &format!(
            "inline constexpr uint64_t marker_ids[]={{{}}};\n",
            compiled
                .markers
                .iter()
                .map(|(_, id)| format!(
                    "UINT64_C({})",
                    crate::blueprint_refs::compact_id(&id.to_string())
                ))
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    let signals = compiled.markers.len() + compiled.events.len();
    out += &format!(
        "inline const Asset asset={{UINT64_C({}),{},{},{},{},{},{},{signals},{},{},{},{},{}}};\n}}\n",
        crate::blueprint_refs::compact_id(&compiled.asset.to_string()),
        compiled.duration_ticks,
        compiled.loop_mode == crate::timeline::LoopMode::Repeat,
        compiled.slots.len(),
        compiled.tracks.len(),
        compiled.events.len(),
        compiled.markers.len(),
        if compiled.slots.is_empty() {
            "nullptr"
        } else {
            "targets"
        },
        if compiled.tracks.is_empty() {
            "nullptr"
        } else {
            "properties"
        },
        if compiled.events.is_empty() {
            "nullptr"
        } else {
            "events"
        },
        if compiled.markers.is_empty() {
            "nullptr"
        } else {
            "marker_ids"
        },
        if signals == 0 {
            "nullptr".into()
        } else {
            format!("signals_{stem}")
        }
    );
    Ok(out)
}
