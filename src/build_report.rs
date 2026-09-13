//! Sizes and analysis belonging to one successful build, including cached builds.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub generate_asset_report: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            generate_asset_report: true,
        }
    }
}
pub fn directory(root: &Path, debug: bool) -> PathBuf {
    root.join(if debug {
        ".epok/build-blueprint-debug"
    } else {
        ".epok/build"
    })
}
const SUMMARY: &str = "build-summary.json";
const INPUTS: [&str; 4] = ["epok.ps-exe", "epok.elf", "epok.map", "memory-inputs.json"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Summary {
    pub profile: crate::play::Profile,
    pub debug: bool,
    pub request: Option<String>,
    pub exe_bytes: u64,
    pub external_bytes: u64,
    pub disc_bytes: Option<u64>,
    pub automatic_report: bool,
    pub report_hash: Option<String>,
    inputs: BTreeMap<String, String>,
}
impl Summary {
    pub fn capture(
        build: &Path,
        profile: crate::play::Profile,
        debug: bool,
        request: Option<String>,
        disc: Option<&Path>,
        automatic_report: bool,
    ) -> Result<Self, String> {
        let mut inputs = BTreeMap::new();
        let mut paths: Vec<String> = INPUTS.iter().map(|p| (*p).into()).collect();
        paths.extend(crate::memory::external_file_paths(build)?);
        if let Some(disc) = disc {
            paths.push(disc.file_name().unwrap().to_string_lossy().into_owned());
            if disc.extension().is_some_and(|ext| ext == "cue") {
                paths.push("epok.bin".into());
            }
        }
        for path in paths {
            inputs.insert(
                path.clone(),
                crate::assets::hash(&std::fs::read(build.join(path)).map_err(|e| e.to_string())?),
            );
        }
        let exe_bytes = std::fs::metadata(build.join("epok.ps-exe"))
            .map_err(|e| e.to_string())?
            .len();
        let disc_bytes = disc
            .map(|path| {
                let image = if path.extension().is_some_and(|ext| ext == "cue") {
                    path.with_extension("bin")
                } else {
                    path.to_owned()
                };
                std::fs::metadata(image)
                    .map(|m| m.len())
                    .map_err(|e| e.to_string())
            })
            .transpose()?;
        Ok(Self {
            profile,
            debug,
            request,
            exe_bytes,
            external_bytes: crate::memory::external_file_bytes(build)?,
            disc_bytes,
            automatic_report,
            report_hash: None,
            inputs,
        })
    }
    pub fn load(root: &Path, debug: bool) -> Option<Self> {
        let summary: Self =
            serde_json::from_slice(&std::fs::read(directory(root, debug).join(SUMMARY)).ok()?)
                .ok()?;
        (summary.debug == debug && INPUTS.iter().all(|p| summary.inputs.contains_key(*p)))
            .then_some(summary)
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        crate::project::write_changed(
            &directory(root, self.debug).join(SUMMARY),
            &serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
    }
    pub fn verify_inputs(&self, root: &Path) -> Result<(), String> {
        for (path, expected) in &self.inputs {
            let allowed = INPUTS.contains(&path.as_str())
                || matches!(
                    path.as_str(),
                    "GEOMETRY.BIN" | "epok.cue" | "epok.bin" | "epok.iso"
                )
                || path
                    .strip_prefix("music/M")
                    .and_then(|p| p.strip_suffix(".XA"))
                    .is_some_and(|n| n.len() == 7 && n.bytes().all(|b| b.is_ascii_digit()));
            if !allowed {
                return Err("Invalid report snapshot path".into());
            }
            let bytes =
                std::fs::read(directory(root, self.debug).join(path)).map_err(|e| e.to_string())?;
            if *expected != crate::assets::hash(&bytes) {
                return Err(
                    "Compiled analysis inputs changed. Build again before generating a report."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn record_report(&mut self, root: &Path) -> Result<(), String> {
        self.report_hash = Some(crate::assets::hash(
            &std::fs::read(directory(root, self.debug).join("memory-report.json"))
                .map_err(|e| e.to_string())?,
        ));
        Ok(())
    }
    pub fn report(&self, root: &Path) -> Option<crate::memory::Report> {
        let bytes = std::fs::read(directory(root, self.debug).join("memory-report.json")).ok()?;
        if self.report_hash.as_ref()? != &crate::assets::hash(&bytes) {
            return None;
        }
        let report: crate::memory::Report = serde_json::from_slice(&bytes).ok()?;
        (self.inputs.get("epok.ps-exe") == Some(&report.executable_hash)
            && report.profile == self.profile
            && report.debug == self.debug)
            .then_some(report)
    }
    pub fn status(&self) -> String {
        use crate::play::{DataSource, Target};
        let size = crate::memory_ui::size;
        let exe = if self.profile.target == Target::Serial {
            "Serial EXE"
        } else {
            "EXE"
        };
        let data = match self.profile.data {
            DataSource::Executable => "Assets: in EXE".into(),
            DataSource::Host => format!("PC assets: {}", size(self.external_bytes)),
            DataSource::Disc => format!("CD assets: {}", size(self.external_bytes)),
        };
        let disc = self
            .disc_bytes
            .map(|bytes| format!(" | Disc image: {}", size(bytes)))
            .unwrap_or_default();
        format!("{exe}: {} | {data}{disc}", size(self.exe_bytes))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub fn fixture(root: &Path, automatic: bool, with_report: bool) -> Summary {
        let build = directory(root, false);
        std::fs::create_dir_all(&build).unwrap();
        for name in INPUTS {
            std::fs::write(build.join(name), b"PS-X EXE fixture").unwrap();
        }
        let external = crate::memory::Node::leaf("GEOMETRY.BIN", 128, "external");
        std::fs::write(build.join("GEOMETRY.BIN"), [0; 128]).unwrap();
        std::fs::write(
            build.join("memory-inputs.json"),
            serde_json::to_vec(
                &serde_json::json!({"hints":[],"scenes":[],"spu":[],"files":[external]}),
            )
            .unwrap(),
        )
        .unwrap();
        let mut summary = Summary::capture(
            &build,
            crate::play::Profile::default(),
            false,
            Some("request".into()),
            None,
            automatic,
        )
        .unwrap();
        if with_report {
            let empty = || crate::memory::Space {
                capacity: None,
                used: 0,
                root: Default::default(),
            };
            let report = crate::memory::Report {
                profile: summary.profile.clone(),
                debug: false,
                executable_hash: summary.inputs["epok.ps-exe"].clone(),
                ram: empty(),
                scratchpad: empty(),
                scenes: Vec::new(),
                spu: empty(),
                files: empty(),
                warnings: Vec::new(),
            };
            std::fs::write(
                build.join("memory-report.json"),
                serde_json::to_vec(&report).unwrap(),
            )
            .unwrap();
            summary.record_report(root).unwrap();
        }
        summary.save(root).unwrap();
        summary
    }
    #[test]
    fn build_sizes_and_report_are_bound_to_the_compiled_snapshot() {
        let root = crate::workspace::tests::temp("build-report-snapshot");
        let summary = fixture(&root, true, true);
        assert_eq!(summary.external_bytes, 128);
        assert_eq!(summary.exe_bytes, 16);
        assert!(summary.report(&root).is_some());
        assert!(
            Summary::load(&root, false)
                .unwrap()
                .verify_inputs(&root)
                .is_ok()
        );
        std::fs::write(
            directory(&root, false).join("memory-inputs.json"),
            b"changed staging",
        )
        .unwrap();
        assert!(summary.verify_inputs(&root).is_err());
        assert!(
            summary.report(&root).is_some(),
            "The previous report stays viewable after sources/staging change"
        );
        std::fs::write(directory(&root, false).join("memory-report.json"), b"{}").unwrap();
        assert!(summary.report(&root).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn transport_labels_do_not_double_count_resident_assets() {
        let root = crate::workspace::tests::temp("build-report-labels");
        let mut summary = fixture(&root, false, false);
        assert!(summary.status().contains("Assets: in EXE"));
        summary.profile.target = crate::play::Target::Serial;
        summary.profile.data = crate::play::DataSource::Host;
        assert!(
            summary
                .status()
                .contains("Serial EXE: 16 B | PC assets: 128 B")
        );
        summary.profile.target = crate::play::Target::Embedded;
        summary.profile.data = crate::play::DataSource::Disc;
        summary.disc_bytes = Some(4096);
        assert!(
            summary
                .status()
                .contains("CD assets: 128 B | Disc image: 4.0 KiB")
        );
        assert!(Options::default().generate_asset_report);
        assert!(
            serde_json::from_str::<Options>("{}")
                .unwrap()
                .generate_asset_report
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
