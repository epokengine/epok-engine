//! Typed adapters for existing source assets. Nodes specialize by asset UUID;
//! slot UUIDs become pin identities and cooked indices, never runtime lookups.
use crate::{
    blueprint_asset::{AssetFile, Builtin, NodeKind},
    reflection_schema::Type,
    timeline::{Slot, TimelineAsset},
};
use std::{collections::BTreeSet, path::PathBuf};
use uuid::Uuid;

pub fn references(files: &[AssetFile]) -> Result<(BTreeSet<Uuid>, BTreeSet<Uuid>), String> {
    let mut timelines = BTreeSet::new();
    let mut effects = BTreeSet::new();
    for file in files {
        for graph in &file.asset.functions {
            for node in graph.compilation_nodes() {
                if let NodeKind::Builtin { operation } = &node.kind {
                    let (value, output) = match operation {
                        Builtin::PlayTimelineAsset { asset } => (asset, &mut timelines),
                        Builtin::SpawnParticleEffect { asset } => (asset, &mut effects),
                        _ => continue,
                    };
                    let id = Uuid::parse_str(value)
                        .map_err(|_| format!("Playback asset reference {value} must be a UUID"))?;
                    if id.is_nil() || id.to_string() != *value {
                        return Err(format!(
                            "Playback asset reference {value} must be a canonical non-nil UUID"
                        ));
                    }
                    output.insert(id);
                }
            }
        }
    }
    Ok((timelines, effects))
}
pub fn slots<'a>(
    operation: &Builtin,
    timelines: &'a [(PathBuf, TimelineAsset)],
    effects: &'a [(PathBuf, crate::particle_effect::ParticleEffect)],
) -> Result<Option<&'a [Slot]>, String> {
    match operation {
        Builtin::PlayTimelineAsset { asset } => timelines
            .iter()
            .find(|(path, timeline)| {
                path.to_string_lossy().ends_with(".timeline.json")
                    && timeline.id.to_string() == *asset
            })
            .map(|(_, asset)| Some(asset.slots.as_slice()))
            .ok_or_else(|| {
                format!("Missing standalone TimelineAsset {asset}; source reference preserved")
            }),
        Builtin::SpawnParticleEffect { asset } => effects
            .iter()
            .find(|(_, effect)| effect.id.to_string() == *asset)
            .map(|(_, effect)| Some(effect.timeline.slots.as_slice()))
            .ok_or_else(|| format!("Missing ParticleEffect {asset}; source reference preserved")),
        _ => Ok(None),
    }
}
pub fn external(slots: &[Slot]) -> Vec<&Slot> {
    let mut result = slots
        .iter()
        .filter(|slot| {
            matches!(
                slot.target,
                Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
            )
        })
        .collect::<Vec<_>>();
    result.sort_by_key(|slot| slot.id);
    result
}
pub fn pin(slot: &Slot) -> String {
    format!("binding:{}", slot.id)
}
pub fn call_name(operation: &Builtin) -> Option<String> {
    let (prefix, asset) = match operation {
        Builtin::PlayTimelineAsset { asset } => ("sequence", asset),
        Builtin::SpawnParticleEffect { asset } => ("effect", asset),
        _ => return None,
    };
    Uuid::parse_str(asset)
        .ok()
        .map(|id| format!("epok::bp::playback::{prefix}_{}", id.simple()))
}
pub fn parameters(slots: &[Slot], effect: bool) -> String {
    let mut parameters = if effect {
        vec![
            "const epok::Transform& transform".into(),
            "uint32_t seed".into(),
            "epok::ObjectId owner".into(),
        ]
    } else {
        vec!["epok::ObjectId owner".into()]
    };
    parameters.extend(
        external(slots)
            .into_iter()
            .map(|slot| format!("epok::ObjectId slot_{}", slot.id.simple())),
    );
    parameters.join(",")
}
pub fn declaration(id: Uuid, slots: &[Slot], effect: bool) -> String {
    format!(
        "{}::Handle {}_{}({});\n",
        if effect {
            "epok::effects"
        } else {
            "epok::timeline"
        },
        if effect { "effect" } else { "sequence" },
        id.simple(),
        parameters(slots, effect)
    )
}
/// Definitions follow cooked asset includes after all native/Blueprint types.
/// Existing source loaders, cookers and runtime services remain authoritative.
pub fn definitions(
    timelines: &[crate::timeline_scene::Prepared],
    effects: &[crate::particle_effect_scene::Prepared],
) -> String {
    let mut out = String::from("namespace epok::bp::playback {\n");
    for (id, slots, compiled, effect) in timelines
        .iter()
        .map(|asset| (asset.source.id, &asset.source.slots, &asset.compiled, false))
        .chain(effects.iter().map(|asset| {
            (
                asset.source.id,
                &asset.source.timeline.slots,
                &asset.compiled,
                true,
            )
        }))
    {
        let targets = compiled
            .slots
            .iter()
            .map(|slot| {
                if matches!(
                    slot.target,
                    Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
                ) {
                    format!("epok::timeline::BoundTarget(slot_{})", slot.id.simple())
                } else {
                    "epok::timeline::BoundTarget{}".into()
                }
            })
            .collect::<Vec<_>>();
        let array = if targets.is_empty() {
            String::new()
        } else {
            format!(
                "const epok::timeline::BoundTarget targets[]={{{}}};",
                targets.join(",")
            )
        };
        let pointer = if targets.is_empty() {
            "nullptr"
        } else {
            "targets"
        };
        let service = if effect { "effects" } else { "timeline" };
        let invoke = if effect {
            format!(
                "epok::effects::spawn(epok::effects::cooked::asset_{}::asset,transform,seed,epok::bp::data_handle(owner),{pointer})",
                id.simple()
            )
        } else {
            format!(
                "epok::timeline::sequences.play(epok::timeline::cooked::asset_{}::asset,owner,{pointer},epok::blueprint_scene_generation)",
                id.simple()
            )
        };
        out += &format!(
            "epok::{service}::Handle {}_{}({}){{{array}const auto result={invoke};epok::{service}::publish_stats();return result;}}\n",
            if effect { "effect" } else { "sequence" },
            id.simple(),
            parameters(slots, effect)
        );
    }
    out + "}\n"
}
