//! Dependency footprints of successfully generated Lua classes. Mirrors
//! `blueprint_dependencies`, with the execution mode folded into every
//! generated node so an AOT header can never survive into a VM build.
use crate::{
    artifact_dependencies::{self, Graph},
    blueprint::Registry,
    lua_asset::LuaFile,
    lua_compile::Compilation,
    reflection_schema as schema,
    settings::{LUA_FRONTEND_VERSION, LUA_PROFILE_VERSION, LuaExecution},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Bumped whenever the emitted C++ changes shape without a profile change.
pub const COMPILER_VERSION: u32 = 1;

fn hash(value: impl serde::Serialize) -> String {
    crate::assets::hash(&serde_json::to_vec(&value).expect("Serializable dependency"))
}

/// Line-ending independent hash of the authored text. A `.lua` file has no
/// canonical document form, so its bytes are its semantic identity.
pub fn semantic(file: &LuaFile) -> String {
    crate::assets::hash(file.source.replace("\r\n", "\n").as_bytes())
}

/// Membership of the compiler input list, including duplicates.
pub fn source_set(files: &[LuaFile]) -> String {
    let mut paths = files
        .iter()
        .map(|file| crate::lua_aot::project_relative(&file.path))
        .collect::<Vec<_>>();
    paths.sort();
    hash(paths)
}

/// Reflected contract a generated Lua class was compiled against.
pub fn reflected(key: &str, registry: &Registry) -> Option<String> {
    let (kind, id) = key.split_once(':')?;
    if kind != "lua-class" {
        return None;
    }
    let c = registry.classes.get(id)?;
    Some(hash(serde_json::json!([
        c.id,
        c.cpp_name,
        c.parent,
        c.provider,
        c.backend,
        c.abstract_class,
        c.final_class,
        c.blueprintable,
        c.properties
            .iter()
            .map(|p| serde_json::json!([p.id, p.name, p.value_type, p.default, p.editable]))
            .collect::<Vec<_>>(),
        c.functions
            .iter()
            .map(|f| serde_json::json!([
                f.id,
                f.name,
                f.parameters,
                f.returns,
                f.callable,
                f.event,
                f.pure,
                f.abstract_method,
                f.final_method,
                f.access,
                f.overrides
            ]))
            .collect::<Vec<_>>()
    ])))
}

/// Best-effort class identity per source. Observation must never certify a
/// file whose declaration does not currently extract.
fn identities(files: &[LuaFile]) -> BTreeMap<String, &LuaFile> {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut resolved = BTreeMap::new();
    for file in files {
        if let Ok(declaration) = crate::lua_asset::extract(file) {
            *counts.entry(declaration.id.clone()).or_default() += 1;
            resolved.insert(declaration.id, file);
        }
    }
    resolved.retain(|id, _| counts[id] == 1);
    resolved
}

pub fn observe_sources(root: &Path, files: &[LuaFile]) -> Result<(), String> {
    let current = identities(files)
        .into_iter()
        .map(|(id, file)| (format!("lua:{id}"), semantic(file)))
        .collect::<BTreeMap<_, _>>();
    artifact_dependencies::transaction(root, |graph| {
        graph.publish("lua-sources", source_set(files), BTreeSet::new());
        let removed = graph
            .nodes
            .keys()
            .filter(|key| key.starts_with("lua:") && !current.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for key in removed {
            graph.invalidate(&key, "Lua source was removed or has duplicate identity");
        }
        for (key, signature) in current {
            graph.publish(&key, signature, BTreeSet::new());
        }
    })
    .map(|_| ())
}

pub fn observe_reflection(root: &Path, registry: &Registry) -> Result<(), String> {
    artifact_dependencies::transaction(root, |graph| {
        let keys = graph
            .nodes
            .keys()
            .filter(|key| key.starts_with("lua-class:"))
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            match reflected(&key, registry) {
                Some(signature) => graph.publish(&key, signature, BTreeSet::new()),
                None => graph.invalidate(
                    &key,
                    "Reflected Lua dependency was removed or is incompatible",
                ),
            }
        }
    })
    .map(|_| ())
}

pub fn invalidate(root: &Path, files: &[LuaFile], reason: &str) -> Result<(), String> {
    let ids = identities(files).into_keys().collect::<Vec<_>>();
    artifact_dependencies::transaction(root, |graph| {
        for id in &ids {
            graph.invalidate(&format!("lua:{id}"), reason);
            graph.invalidate(&format!("generated-lua:{id}"), reason);
        }
    })
    .map(|_| ())
}

pub fn invalidate_all(root: &Path, reason: &str) -> Result<(), String> {
    artifact_dependencies::transaction(root, |graph| {
        let keys = graph
            .nodes
            .keys()
            .filter(|key| {
                key.starts_with("lua:")
                    || key.starts_with("generated-lua:")
                    || key.starts_with("lua-class:")
                    || *key == "lua-sources"
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            graph.invalidate(&key, reason);
        }
    })
    .map(|_| ())
}

pub type Footprints = BTreeMap<String, (String, BTreeMap<String, String>)>;

/// Capture the exact snapshots that produced the generated code. The sources
/// are not read again here: that could certify old code against a new revision.
pub fn capture(
    files: &[(String, &LuaFile)],
    registry: &Registry,
    artifacts: &crate::script_backend::Artifacts,
    mode: LuaExecution,
) -> Result<Footprints, String> {
    let shared = artifacts
        .files
        .get(Path::new("scripts/generated/lua/lua_bindings.cpp"))
        .map(|data| crate::assets::hash(data));
    let mut footprints = Footprints::new();
    for (id, file) in files {
        let class = registry
            .classes
            .get(id)
            .ok_or_else(|| format!("Lua class {id} was not published"))?;
        let mut inputs = BTreeMap::from([
            (format!("lua:{id}"), semantic(file)),
            ("lua-compiler".into(), COMPILER_VERSION.to_string()),
            ("lua-mode".into(), mode.signature()),
            ("lua-profile".into(), LUA_PROFILE_VERSION.to_string()),
            ("lua-frontend".into(), LUA_FRONTEND_VERSION.to_string()),
            (
                "reflection-schema".into(),
                schema::SCHEMA_VERSION.to_string(),
            ),
        ]);
        // The whole ancestry is a compile input: a base field or signature
        // change relayouts or breaks this subclass.
        for ancestor in registry.ancestry(&class.cpp_name) {
            let key = format!("lua-class:{}", ancestor.id);
            inputs.insert(
                key.clone(),
                reflected(&key, registry).expect("Known Lua ancestry"),
            );
            if let Some((_, parent)) = files.iter().find(|(id, _)| *id == ancestor.id) {
                inputs.insert(format!("lua:{}", ancestor.id), semantic(parent));
            }
        }
        let path = format!(
            "scripts/generated/lua/{}.hpp",
            crate::lua_asset::artifact_stem(id)
        );
        let mut outputs = BTreeMap::from([(
            path.clone(),
            artifacts
                .files
                .get(Path::new(&path))
                .map(|data| crate::assets::hash(data))
                .ok_or_else(|| format!("Lua class {id} is missing its generated header"))?,
        )]);
        if let Some(bindings) = &shared {
            outputs.insert(
                "scripts/generated/lua/lua_bindings.cpp".into(),
                bindings.clone(),
            );
        }
        footprints.insert(format!("generated-lua:{id}"), (hash(outputs), inputs));
    }
    Ok(footprints)
}

pub fn record(root: &Path, compiled: &Compilation) -> Result<(), String> {
    observe_reflection(root, &compiled.registry)?;
    artifact_dependencies::transaction(root, |graph: &mut Graph| {
        for (_, inputs) in compiled.footprints.values() {
            for (id, signature) in inputs {
                graph.publish(id, signature.clone(), BTreeSet::new());
            }
        }
        for (id, (signature, inputs)) in &compiled.footprints {
            graph.publish(id, signature.clone(), inputs.keys().cloned().collect());
        }
    })
    .map(|_| ())
}
