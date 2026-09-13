//! Provenance for generated playback and scene headers in a staging directory.
//! Asset identity still belongs to the existing source loaders and compilers.
use crate::{artifact_dependencies, particle_effect_scene, timeline_compile};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};

/// Exact successful provider output and the existing input signatures it used.
/// This carries provenance only; providers still own loading and conversion.
#[derive(Debug)]
pub struct ResourceOutput {
    pub path: String,
    pub signature: String,
    pub inputs: BTreeMap<String, String>,
}

pub fn target(root: &Path, destination: &Path) -> Result<String, String> {
    let relative = destination
        .strip_prefix(root)
        .map_err(|_| "Playback staging destination must be inside the project")?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Playback staging requires a relative output directory".into());
    }
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

#[derive(Default)]
pub struct Batch {
    target: String,
    observed: BTreeMap<String, artifact_dependencies::Node>,
    inputs: BTreeMap<String, String>,
    cooked: BTreeMap<String, (String, BTreeSet<String>)>,
    outputs: BTreeMap<String, (String, BTreeSet<String>)>,
    scene_inputs: BTreeSet<String>,
    audio_inputs: BTreeSet<String>,
    staged: Option<crate::staging_files::Files>,
}
impl Batch {
    pub fn new(root: &Path, destination: &Path) -> Result<Self, String> {
        Ok(Self {
            target: target(root, destination)?,
            observed: artifact_dependencies::Graph::load(root)?.nodes,
            ..Self::default()
        })
    }
    fn input(&mut self, id: String, signature: String) -> Result<String, String> {
        if self.inputs.get(&id).is_some_and(|old| *old != signature) {
            return Err(format!(
                "Playback dependency {id} changed during staging; retry the build"
            ));
        }
        self.inputs.insert(id.clone(), signature);
        Ok(id)
    }
    fn compiled(&mut self, compiled: &timeline_compile::Compiled) -> Result<String, String> {
        for (id, signature) in &compiled.dependencies {
            self.input(id.clone(), signature.clone())?;
        }
        let id = format!("cooked-timeline:{}", compiled.asset);
        self.cooked.insert(
            id.clone(),
            (
                compiled.signature.clone(),
                compiled.dependencies.keys().cloned().collect(),
            ),
        );
        Ok(id)
    }
    fn output(&mut self, path: String, bytes: &[u8], dependencies: BTreeSet<String>) {
        self.outputs.insert(
            format!("generated-playback:{}/{path}", self.target),
            (crate::assets::hash(bytes), dependencies),
        );
    }
    pub fn scene_source(
        &mut self,
        root: &Path,
        source: &crate::scene_dependencies::Origin,
    ) -> Result<(), String> {
        let (id, signature) = source.dependency(root, &self.target)?;
        self.scene_input(id, signature)
    }
    pub fn scene_input(&mut self, id: String, signature: String) -> Result<(), String> {
        let id = self.input(id, signature)?;
        self.scene_inputs.insert(id);
        Ok(())
    }
    pub fn scene_catalog(
        &mut self,
        root: &Path,
        catalog: &[crate::scripts::Script],
    ) -> Result<(), String> {
        self.scene_input(
            "scene-catalog".into(),
            crate::scene_dependencies::catalog_signature(catalog),
        )?;
        let key = self.input(
            "audio-catalog".into(),
            crate::audio::catalog_selection_signature(root, catalog),
        )?;
        self.audio_inputs.insert(key);
        Ok(())
    }
    pub fn scene_audio_source(
        &mut self,
        root: &Path,
        origin: &crate::scene_dependencies::Origin,
        scene: &crate::scene::Scene,
        registry: &crate::blueprint::Registry,
    ) -> Result<(), String> {
        let (source, _) = origin.dependency(root, &self.target)?;
        let key = self.input(
            format!("audio-selection:{source}"),
            crate::audio::selection_signature(scene, Some(registry)),
        )?;
        self.audio_inputs.insert(key);
        Ok(())
    }
    pub fn timeline_audio_source(
        &mut self,
        source: &crate::timeline::TimelineAsset,
    ) -> Result<(), String> {
        self.input(format!("timeline:{}", source.id), source.semantic_hash())?;
        let key = self.input(
            format!("timeline-audio:{}", source.id),
            crate::audio::timeline_selection_signature(source),
        )?;
        self.audio_inputs.insert(key);
        Ok(())
    }
    pub fn blueprint_audio_source(
        &mut self,
        file: &crate::blueprint_asset::AssetFile,
        registry: Option<&crate::blueprint::Registry>,
    ) -> Result<(), String> {
        self.input(
            format!("blueprint:{}", file.asset.id),
            file.asset.semantic_hash(),
        )?;
        let key = self.input(
            format!("blueprint-audio:{}", file.asset.id),
            crate::audio::blueprint_selection_signature(file, registry),
        )?;
        self.audio_inputs.insert(key);
        Ok(())
    }
    /// Components, timelines, Blueprint graphs and resolved catalog declarations
    /// have audio selection projections, including typed inherited defaults.
    /// Fresh declaration observation covers native metadata and script values;
    /// raw native code changes remain stage/build inputs, not audio-bank inputs.
    pub fn audio_bank_inputs(&mut self) {
        let mut inputs = self.audio_inputs.clone();
        inputs.extend(
            self.scene_inputs
                .iter()
                .filter(|key| matches!(key.as_str(), "scene-registry" | "scene-inventory" | "scene-play-settings" | "blueprint-sources"))
                .cloned(),
        );
        if let Some((_, dependencies)) = self
            .outputs
            .get_mut(&format!("generated-resource:{}/audio-bank.hh", self.target))
        {
            dependencies.extend(inputs);
        }
    }
    pub fn scene_resources(
        &mut self,
        scene: &crate::scene::Scene,
        index: &crate::assets::Index,
    ) -> Result<(), String> {
        let mut ids = crate::texture::ids(scene)
            .into_iter()
            .collect::<BTreeSet<_>>();
        ids.extend(crate::audio::clip_ids(scene));
        for entity in &scene.entities {
            if let Some(mesh) = &entity.editable_mesh {
                ids.insert(mesh.asset);
            }
            if let Some(mesh) = &entity.skeletal_mesh {
                ids.insert(mesh.asset);
                ids.extend(mesh.clip);
                if let Some(model) = &mesh.model {
                    ids.extend(model.clips.iter().map(|(id, _)| *id));
                }
            }
        }
        for id in ids {
            self.scene_input(
                format!("asset:{id}"),
                crate::assets::cache_key(&index.resolve(id)?.meta),
            )?;
        }
        Ok(())
    }
    pub fn scene_header(&mut self, bytes: &[u8]) {
        let mut dependencies = self.scene_inputs.clone();
        dependencies.extend(
            self.outputs
                .keys()
                .filter(|id| {
                    id.starts_with("generated-playback:")
                        || (id.starts_with("generated-script:")
                            && [".hpp", ".hh", ".h", ".inl"]
                                .iter()
                                .any(|suffix| id.ends_with(suffix)))
                })
                .cloned(),
        );
        self.outputs.insert(
            format!("generated-scene:{}/scene.hh", self.target),
            (crate::assets::hash(bytes), dependencies),
        );
    }
    pub fn resources(&mut self, resources: Vec<ResourceOutput>) -> Result<(), String> {
        for resource in resources {
            let mut dependencies = BTreeSet::new();
            for (key, signature) in resource.inputs {
                dependencies.insert(self.input(key, signature)?);
            }
            self.outputs.insert(
                format!("generated-resource:{}/{}", self.target, resource.path),
                (resource.signature, dependencies),
            );
        }
        Ok(())
    }
    pub fn scripts(&mut self, artifacts: &crate::script_backend::Artifacts) -> Result<(), String> {
        if let Some(signature) = &artifacts.native_set {
            self.input("native-sources".into(), signature.clone())?;
        }
        if let Some(signature) = &artifacts.blueprint_set {
            self.scene_input("blueprint-sources".into(), signature.clone())?;
        }
        for (key, (signature, inputs)) in &artifacts.blueprint_footprints {
            for (id, value) in inputs {
                self.input(id.clone(), value.clone())?;
            }
            self.cooked.insert(
                key.clone(),
                (signature.clone(), inputs.keys().cloned().collect()),
            );
        }
        for (path, bytes) in &artifacts.files {
            let path_string = path.to_string_lossy().replace('\\', "/");
            let dependencies = if let Some((key, signature)) = artifacts.native_inputs.get(path) {
                BTreeSet::from([self.input(key.clone(), signature.clone())?])
            } else {
                let class = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|id| format!("generated-blueprint:{id}"))
                    .filter(|key| artifacts.blueprint_footprints.contains_key(key));
                class.map_or_else(
                    || {
                        let mut inputs = artifacts
                            .blueprint_footprints
                            .keys()
                            .cloned()
                            .collect::<BTreeSet<_>>();
                        if artifacts.blueprint_set.is_some() {
                            inputs.insert("blueprint-sources".into());
                        }
                        inputs
                    },
                    |key| BTreeSet::from([key]),
                )
            };
            self.outputs.insert(
                format!("generated-script:{}/{path_string}", self.target),
                (crate::assets::hash(bytes), dependencies),
            );
        }
        Ok(())
    }
    /// Include precisely the files handed to the staging writer, even when an
    /// unchanged write was skipped. Old object files and removed sources cannot
    /// enter this manifest just because they remain in the output directory.
    pub fn files(&mut self, files: crate::staging_files::Files) -> Result<(), String> {
        for (path, signature) in &files {
            let known = [
                "generated-playback",
                "generated-scene",
                "generated-script",
                "generated-resource",
            ]
            .iter()
            .map(|kind| format!("{kind}:{}/{path}", self.target))
            .find(|key| self.outputs.contains_key(key));
            let dependencies = if let Some(key) = known {
                if self.outputs[&key].0 != *signature {
                    return Err(format!(
                        "Staged output {path} differs from its captured generation"
                    ));
                }
                BTreeSet::from([key])
            } else if crate::project::runtime_sources()
                .iter()
                .any(|(p, _)| *p == path)
                || ["text.hpp", "hud-font.hh"].contains(&path.as_str())
            {
                BTreeSet::from([self.input(format!("runtime-file:{path}"), signature.clone())?])
            } else {
                // These resources are generated from the validated scene and
                // shared resource selection. Detailed provider footprints can
                // narrow this set without changing the manifest/cook contract.
                let mut inputs = self.scene_inputs.clone();
                inputs.extend(
                    self.outputs
                        .keys()
                        .filter(|key| key.starts_with("generated-playback:"))
                        .cloned(),
                );
                if path == "sources.mk" || path == "Scripts.epokmanifest" {
                    inputs.insert("native-sources".into());
                    inputs.extend(
                        self.outputs
                            .keys()
                            .filter(|key| key.starts_with("generated-script:"))
                            .cloned(),
                    );
                }
                inputs
            };
            self.outputs.insert(
                format!("staged-file:{}/{path}", self.target),
                (signature.clone(), dependencies),
            );
        }
        self.staged = Some(files);
        Ok(())
    }
    pub fn timeline(
        &mut self,
        compiled: &timeline_compile::Compiled,
        bytes: &[u8],
    ) -> Result<(), String> {
        let dependencies = BTreeSet::from([self.compiled(compiled)?]);
        self.output(
            format!("timelines/{}.hh", compiled.asset),
            bytes,
            dependencies,
        );
        Ok(())
    }
    pub fn effect(
        &mut self,
        prepared: &particle_effect_scene::Prepared,
        resources: &crate::scene::Scene,
        index: &crate::assets::Index,
        bytes: &[u8],
    ) -> Result<(), String> {
        let effect = &prepared.source;
        let mut dependencies = BTreeSet::from([
            self.compiled(&prepared.compiled)?,
            self.input(format!("effect:{}", effect.id), effect.semantic_hash())?,
        ]);
        let texture_ids = crate::texture::ids(resources);
        for (_, value) in particle_effect_scene::resources(std::slice::from_ref(prepared)) {
            let id: uuid::Uuid = serde_json::from_value(value).map_err(|e| e.to_string())?;
            let record = index.resolve(id)?;
            dependencies.insert(self.input(
                format!("asset:{id}"),
                crate::assets::cache_key(&record.meta),
            )?);
            let position = texture_ids
                .iter()
                .position(|candidate| *candidate == id)
                .ok_or_else(|| format!("Effect {} has no staged texture {id}", effect.id))?;
            // Only mappings used by this header matter. Adding a texture after
            // them in the shared table must not invalidate unrelated headers.
            dependencies.insert(self.input(
                format!("texture-binding:{}:{id}", self.target),
                crate::assets::hash(position.to_string().as_bytes()),
            )?);
        }
        // Instance overrides and entity bindings are emitted in scene.hh, not
        // this reusable asset header. Their provenance belongs to scene staging.
        self.output(format!("effects/{}.hh", effect.id), bytes, dependencies);
        Ok(())
    }
    /// Call only after all staging writes succeed. Capture uses compiler/header
    /// snapshots above; publishing never rereads a newer source revision.
    pub fn publish(self, root: &Path) -> Result<(), String> {
        let mut conflict = None;
        artifact_dependencies::transaction(root, |graph| {
            // A watcher or another compiler may observe a newer revision while
            // this worker generates headers. Never overwrite that observation
            // with an older snapshot or clear a newly reported source failure.
            for (id, signature) in &self.inputs {
                if let Some(current) = graph.nodes.get(id)
                    && self.observed.get(id) != Some(current)
                    && (current.signature.as_ref() != Some(signature) || !current.stale.is_empty())
                {
                    conflict = Some(format!(
                        "Playback dependency {id} changed during staging; retry the build"
                    ));
                    return;
                }
            }
            let prefixes = [
                "generated-playback",
                "generated-script",
                "generated-resource",
                "staged-file",
            ]
            .map(|kind| format!("{kind}:{}/", self.target));
            let removed = graph
                .nodes
                .keys()
                .filter(|id| {
                    prefixes.iter().any(|prefix| id.starts_with(prefix))
                        && !self.outputs.contains_key(*id)
                })
                .cloned()
                .collect::<Vec<_>>();
            for id in removed {
                graph.invalidate(&id, "Output is no longer part of this staged target");
            }
            for (id, signature) in self.inputs {
                graph.publish(&id, signature, BTreeSet::new());
            }
            for (id, (signature, dependencies)) in self.cooked {
                graph.publish(&id, signature, dependencies);
            }
            let signatures = self
                .outputs
                .iter()
                .filter(|(id, _)| id.starts_with("generated-playback:"))
                .map(|(id, (signature, _))| (id, signature))
                .collect::<BTreeMap<_, _>>();
            let signature = crate::assets::hash(
                &serde_json::to_vec(&signatures).expect("Serializable signatures"),
            );
            let dependencies = signatures
                .keys()
                .map(|id| (*id).clone())
                .collect::<BTreeSet<_>>();
            // Scene headers can depend on script headers; staged-file wrappers
            // follow all generators. Publish in dependency order, not key order.
            let mut remaining = self.outputs;
            while !remaining.is_empty() {
                let ready = remaining
                    .iter()
                    .filter(|(_, (_, dependencies))| {
                        dependencies.iter().all(|id| !remaining.contains_key(id))
                    })
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                if ready.is_empty() {
                    conflict = Some("Generated staging dependencies contain a cycle".into());
                    return;
                }
                for id in ready {
                    let (signature, dependencies) = remaining.remove(&id).expect("Ready output");
                    graph.publish(&id, signature, dependencies);
                }
            }
            // This is the playback subset, not a certificate for the whole
            // build/export, whose other generated resources have their own work.
            graph.publish(
                &format!("stage-playback:{}", self.target),
                signature,
                dependencies,
            );
            if let Some(files) = self.staged {
                graph.publish(
                    &format!("stage:{}", self.target),
                    crate::scene_dependencies::hash(&files),
                    files
                        .keys()
                        .map(|path| format!("staged-file:{}/{path}", self.target))
                        .collect(),
                );
            }
        })?;
        conflict.map_or(Ok(()), Err)
    }
}

pub fn invalidate(root: &Path, destination: &Path, reason: &str) -> Result<(), String> {
    let target = target(root, destination)?;
    artifact_dependencies::transaction(root, |graph| {
        let prefixes = [
            "generated-playback",
            "generated-script",
            "generated-resource",
            "staged-file",
        ]
        .map(|kind| format!("{kind}:{target}/"));
        let outputs = graph
            .nodes
            .keys()
            .filter(|id| {
                prefixes.iter().any(|prefix| id.starts_with(prefix))
                    || *id == &format!("generated-scene:{target}/scene.hh")
            })
            .cloned()
            .collect::<Vec<_>>();
        for id in outputs {
            graph.invalidate(&id, reason);
        }
        graph.invalidate(&format!("stage-playback:{target}"), reason);
        graph.invalidate(&format!("stage:{target}"), reason);
    })?;
    Ok(())
}
