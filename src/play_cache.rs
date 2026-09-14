//! A successful build's receipt, not a replacement for source validation.
use crate::{
    project::Config,
    scene_dependencies::Input,
    staging_files::{BuildTicket, Files},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const RECEIPT: &str = "PlayBuild.epokcache";
#[derive(Serialize, Deserialize)]
pub struct Receipt {
    version: u32,
    request: String,
    ticket: BuildTicket,
    outputs: Files,
    disc: Option<String>,
}
pub fn request(root: &Path, input: &Input, config: &Config, debug: bool) -> Result<String, String> {
    // Host-side exporters/cookers are part of the compiler too. Invalidate old
    // receipts after an editor rebuild even when embedded C++ did not change.
    // Hash once per editor process, not on every Play click.
    static EDITOR: std::sync::OnceLock<Result<String, String>> = std::sync::OnceLock::new();
    let editor = EDITOR
        .get_or_init(|| {
            std::env::current_exe()
                .and_then(std::fs::read)
                .map(|bytes| crate::assets::hash(&bytes))
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)?;
    Ok(crate::scene_dependencies::hash((
        1u32,
        editor,
        input.origin.dependency(root, "play")?,
        crate::scene_dependencies::signature(&input.scene),
        input
            .editor_override
            .as_ref()
            .map(|(path, scene)| (path, crate::scene_dependencies::signature(scene))),
        &input.play,
        &input.play_settings_signature,
        config,
        debug,
        crate::project::fingerprint(root)?,
        crate::project::runtime_sources()
            .iter()
            .map(|(path, bytes)| (*path, crate::assets::hash(bytes)))
            .collect::<Vec<_>>(),
    )))
}
impl Receipt {
    pub fn load(build: &Path, request: &str) -> Option<Self> {
        let receipt: Self =
            crate::document::from_slice(&std::fs::read(build.join(RECEIPT)).ok()?).ok()?;
        if receipt.version != 1 || receipt.request != request {
            return None;
        }
        receipt.verify_outputs(build).ok()?;
        Some(receipt)
    }
    fn verify_outputs(&self, build: &Path) -> Result<(), String> {
        // Bound all paths before reading; a cache is disposable project data.
        for (path, expected) in &self.outputs {
            if !matches!(
                path.as_str(),
                "epok.ps-exe" | "epok.elf" | "epok.map" | "epok.bin" | "epok.cue" | "epok.iso"
            ) {
                return Err("Invalid cached output path".into());
            }
            if crate::assets::hash(&std::fs::read(build.join(path)).map_err(|e| e.to_string())?)
                != *expected
            {
                return Err(format!("Cached output {path} changed"));
            }
        }
        if !self.outputs.contains_key("epok.ps-exe")
            || !self.outputs.contains_key("epok.map")
            || !self.outputs.contains_key("epok.elf")
        {
            return Err("Missing cached outputs".into());
        }
        if let Some(disc) = &self.disc {
            if !matches!(disc.as_str(), "epok.cue" | "epok.iso") || !self.outputs.contains_key(disc)
            {
                return Err("Missing cached disc".into());
            }
            if disc == "epok.cue" && !self.outputs.contains_key("epok.bin") {
                return Err("Missing cached disc data".into());
            }
        }
        Ok(())
    }
    pub fn save(
        build: &Path,
        request: String,
        ticket: BuildTicket,
        disc: Option<&Path>,
    ) -> Result<(), String> {
        let mut outputs = Files::new();
        let mut names = vec!["epok.ps-exe", "epok.elf", "epok.map"];
        let disc = disc.map(|path| path.file_name().unwrap().to_string_lossy().into_owned());
        if let Some(name) = disc.as_deref() {
            names.push(name);
            if name == "epok.cue" {
                names.push("epok.bin");
            }
        }
        for name in names {
            outputs.insert(
                name.into(),
                crate::assets::hash(&std::fs::read(build.join(name)).map_err(|e| e.to_string())?),
            );
        }
        let receipt = Self {
            version: 1,
            request,
            ticket,
            outputs,
            disc,
        };
        crate::project::write_changed(&build.join(RECEIPT), &crate::document::to_vec(&receipt)?)
    }
    pub fn reuse(self, root: &Path, build: &Path) -> Result<(PathBuf, Option<PathBuf>), String> {
        let exe = build.join("epok.ps-exe");
        let bytes = std::fs::read(&exe).map_err(|e| e.to_string())?;
        if !bytes.starts_with(b"PS-X EXE")
            || crate::assets::hash(&bytes) != self.outputs["epok.ps-exe"]
        {
            return Err("Cached executable changed during validation".into());
        }
        let current = BuildTicket::begin_native(root, build)?;
        self.verify_outputs(build)?;
        current.reuse(&self.ticket, root, build, &bytes)?;
        Ok((exe, self.disc.map(|path| build.join(path))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_rejects_changed_missing_and_escaping_outputs() {
        let build = std::env::temp_dir().join(format!("epok-play-cache-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&build).unwrap();
        let ticket = serde_json::from_value(serde_json::json!({
            "target":".epok/build", "stage":crate::artifact_dependencies::Node::default(),
            "options":crate::artifact_dependencies::Node::default(), "native":null, "files":{}
        }))
        .unwrap();
        for name in [
            "epok.ps-exe",
            "epok.elf",
            "epok.map",
            "epok.cue",
            "epok.bin",
        ] {
            std::fs::write(build.join(name), b"original").unwrap();
        }
        Receipt::save(
            &build,
            "request".into(),
            ticket,
            Some(&build.join("epok.cue")),
        )
        .unwrap();
        assert!(Receipt::load(&build, "different request").is_none());
        let mut receipt = Receipt::load(&build, "request").unwrap();
        std::fs::write(build.join("epok.bin"), b"modified").unwrap(); // Same length, raw bytes matter.
        assert!(receipt.verify_outputs(&build).is_err());
        assert!(Receipt::load(&build, "request").is_none());
        std::fs::write(build.join("epok.bin"), b"original").unwrap();
        receipt.outputs.insert("../outside".into(), "hash".into());
        assert!(receipt.verify_outputs(&build).is_err());
        receipt.outputs.remove("../outside");
        receipt.outputs.remove("epok.bin");
        assert!(receipt.verify_outputs(&build).is_err());
        std::fs::remove_dir_all(&build).unwrap();
    }
}
