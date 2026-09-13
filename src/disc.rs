//! Host CD authoring. Mixed XA sectors are finalized by the pinned mkpsxiso tool.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    #[default]
    Scea,
    Scee,
    Scei,
}
impl Region {
    pub const ALL: [Self; 3] = [Self::Scea, Self::Scee, Self::Scei];
    pub fn label(self) -> &'static str {
        match self {
            Self::Scea => "SCEA - North America",
            Self::Scee => "SCEE - Europe / PAL",
            Self::Scei => "SCEI - Japan",
        }
    }
    fn code(self) -> &'static str {
        match self {
            Self::Scea => "SCEA",
            Self::Scee => "SCEE",
            Self::Scei => "SCEI",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    #[default]
    BinCue,
    Iso,
}
impl ImageFormat {
    pub const ALL: [Self; 2] = [Self::BinCue, Self::Iso];
    pub fn label(self) -> &'static str {
        match self {
            Self::BinCue => "BIN / CUE (recommended)",
            Self::Iso => "ISO (burner compatibility)",
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub region: Region,
    pub format: ImageFormat,
    pub license_file: String,
}
impl Settings {
    fn path(root: &Path) -> PathBuf {
        root.join("UserSettings/DiscExport.epokprefs")
    }
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = Self::path(root);
        if !path.is_file() {
            return Ok(Self::default());
        }
        crate::document::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Disc export settings: {e}"))
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        crate::project::write_changed(&Self::path(root), &crate::document::to_vec(self)?)
    }
    pub fn license_path(&self, root: &Path) -> Result<PathBuf, String> {
        let value = self.license_file.trim();
        if value.is_empty() {
            return Err(
                "Choose a PlayStation system-area license file before packaging a physical disc."
                    .into(),
            );
        }
        let path = crate::project::Config::path(root, value);
        if !path.is_file() {
            return Err(format!("License file not found: {}", path.display()));
        }
        Ok(path)
    }
}
fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}
pub fn tool(root: &Path, name: &str) -> Result<PathBuf, String> {
    let config = crate::project::Config::load(root)?;
    let configured = match name {
        "psxavenc" => config.psxavenc,
        "mkpsxiso" => config.mkpsxiso,
        _ => return Err("Unknown audio tool".into()),
    };
    let path = crate::project::Config::executable(root, &configured);
    if crate::dependencies::find_executable(&configured).is_none() {
        return Err(format!(
            "{name} missing at {}. Open Edit > Editor Preferences > Dependencies to configure or install it.",
            path.display()
        ));
    }
    Ok(path)
}
pub fn run(
    command: &mut Command,
    log: &Path,
    mut cancelled: impl FnMut() -> bool,
) -> Result<(), String> {
    let file = fs::File::create(log).map_err(|e| e.to_string())?;
    command
        .stdout(file.try_clone().map_err(|e| e.to_string())?)
        .stderr(file)
        .stdin(Stdio::null());
    crate::pipeline::quiet(command);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let start = Instant::now();
    loop {
        if cancelled() || start.elapsed() > Duration::from_secs(180) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Audio/disc operation cancelled or timed out".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    let mut text = String::new();
                    let _ =
                        fs::File::open(log).and_then(|f| f.take(16384).read_to_string(&mut text));
                    Err(format!("Audio/disc tool failed: {text}"))
                };
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    }
}
pub fn manifest(build: &Path, music: &[String]) -> Result<(), String> {
    manifest_inner(build, music, false)
}
fn manifest_inner(build: &Path, music: &[String], force: bool) -> Result<(), String> {
    let geometry = build.join(crate::streaming::ARCHIVE).is_file();
    if music.is_empty() && !geometry && !force {
        match fs::remove_file(build.join("disc.xml")) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        return Ok(());
    }
    crate::project::write_changed(
        &build.join("SYSTEM.CNF"),
        b"BOOT = cdrom:\\EPOK.EXE;1\r\nTCB = 4\r\nEVENT = 10\r\nSTACK = 801FFF00\r\n",
    )?;
    let mut entries = if geometry {
        format!(
            "<file name=\"{0}\" type=\"data\" source=\"{0}\" date=\"20200101000000\"/>\n",
            crate::streaming::ARCHIVE
        )
    } else {
        String::new()
    };
    entries += &music.iter().map(|name|format!("<file name=\"{name}\" type=\"mixed\" source=\"music/{name}\" date=\"20200101000000\"/>\n")).collect::<String>();
    let xml = format!(
        "<?xml version=\"1.0\"?>\n<iso_project image_name=\"epok.bin\" cue_sheet=\"epok.cue\"><track type=\"data\"><identifiers system=\"PLAYSTATION\" application=\"PLAYSTATION\" volume=\"EPOK\"/><directory_tree><file name=\"SYSTEM.CNF\" source=\"SYSTEM.CNF\" date=\"20200101000000\"/><file name=\"EPOK.EXE\" source=\"epok.ps-exe\" date=\"20200101000000\"/>{entries}</directory_tree></track></iso_project>\n"
    );
    crate::project::write_changed(&build.join("disc.xml"), xml.as_bytes())
}
pub fn physical_manifest(root: &Path, build: &Path, settings: &Settings) -> Result<(), String> {
    let license = settings.license_path(root)?;
    if !build.join("disc.xml").is_file() {
        manifest_inner(build, &[], true)?;
    }
    let source = fs::read_to_string(build.join("disc.xml")).map_err(|e| e.to_string())?;
    let image = match settings.format {
        ImageFormat::BinCue => "image_name=\"epok.bin\" cue_sheet=\"epok.cue\"",
        ImageFormat::Iso => "image_name=\"epok.iso\"",
    };
    let source = source
        .replacen("image_name=\"epok.bin\" cue_sheet=\"epok.cue\"", image, 1)
        .replacen(
            "volume=\"EPOK\"",
            &format!("volume=\"EPOK-{}\"", settings.region.code()),
            1,
        )
        .replacen(
            "<track type=\"data\">",
            &format!(
                "<track type=\"data\"><license file=\"{}\"/>",
                xml(&license.to_string_lossy().replace('\\', "/"))
            ),
            1,
        );
    if !source.contains("<license file=") {
        return Err(
            "Disc manifest does not contain a data track for the system-area license.".into(),
        );
    }
    crate::project::write_changed(&build.join("disc.xml"), source.as_bytes())
}
pub fn build(
    root: &Path,
    folder: &Path,
    cancelled: impl FnMut() -> bool,
) -> Result<Option<PathBuf>, String> {
    if !folder.join("disc.xml").is_file() {
        return Ok(None);
    }
    // Reserve ISO metadata/lead-in space and remain within a 74-minute data disc.
    let mut sectors = 1024
        + fs::metadata(folder.join("epok.ps-exe"))
            .map_err(|e| e.to_string())?
            .len()
            .div_ceil(2048);
    let manifest = fs::read_to_string(folder.join("disc.xml")).map_err(|e| e.to_string())?;
    if manifest.contains(&format!("source=\"{}\"", crate::streaming::ARCHIVE)) {
        let bytes = fs::metadata(folder.join(crate::streaming::ARCHIVE))
            .map_err(|e| e.to_string())?
            .len();
        if bytes == 0 || bytes % crate::streaming::PAGE_BYTES as u64 != 0 {
            return Err("Geometry archive must contain complete 64 KiB pages".into());
        }
        sectors += bytes.div_ceil(2048);
    }
    for entry in fs::read_dir(folder.join("music"))
        .or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                fs::create_dir_all(folder.join("music"))?;
                fs::read_dir(folder.join("music"))
            } else {
                Err(e)
            }
        })
        .map_err(|e| e.to_string())?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        if manifest.contains(&format!(
            "source=\"music/{}\"",
            entry.file_name().to_string_lossy()
        )) {
            sectors += entry
                .metadata()
                .map_err(|e| e.to_string())?
                .len()
                .div_ceil(crate::music::SECTOR as u64);
        }
    }
    if sectors > 74 * 60 * 75 {
        return Err(
            "Game, geometry and audio exceed the 74-minute CD capacity. Reduce referenced resources."
                .into(),
        );
    }
    run(
        Command::new(tool(root, "mkpsxiso")?)
            .current_dir(folder)
            .args(["-y", "-q", "disc.xml"]),
        &folder.join("disc-build.log"),
        cancelled,
    )?;
    let iso = manifest.contains("image_name=\"epok.iso\"");
    let image = folder.join(if iso { "epok.iso" } else { "epok.bin" });
    let size = fs::metadata(&image).map_err(|e| e.to_string())?.len();
    if size < 24 * 2048 {
        return Err("Invalid CD image output".into());
    }
    if iso {
        Ok(Some(image))
    } else {
        let cue = folder.join("epok.cue");
        if !cue.is_file() || size % 2352 != 0 {
            return Err("Invalid BIN/CUE image output".into());
        }
        Ok(Some(cue))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn geometry_only_disc_and_disabled_cleanup() {
        let build = std::env::temp_dir().join(format!("epok-disc-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&build).unwrap();
        std::fs::write(
            build.join(crate::streaming::ARCHIVE),
            vec![0; crate::streaming::PAGE_BYTES],
        )
        .unwrap();
        super::manifest(&build, &[]).unwrap();
        let xml = std::fs::read_to_string(build.join("disc.xml")).unwrap();
        assert!(xml.contains("name=\"GEOMETRY.BIN\" type=\"data\""));
        assert!(build.join("SYSTEM.CNF").exists());
        crate::streaming::clear(&build).unwrap();
        super::manifest(&build, &[]).unwrap();
        assert!(!build.join("disc.xml").exists());
        // The unique temporary directory created by this test owns all files.
        std::fs::remove_dir_all(build).unwrap();
    }

    #[test]
    fn physical_disc_manifest_injects_user_license_and_iso_target() {
        let root =
            std::env::temp_dir().join(format!("epok-physical-disc-{}", uuid::Uuid::new_v4()));
        let build = root.join(".epok/build");
        std::fs::create_dir_all(&build).unwrap();
        let license = root.join("license.dat");
        std::fs::write(&license, [0_u8; 2336]).unwrap();
        super::physical_manifest(
            &root,
            &build,
            &super::Settings {
                region: super::Region::Scee,
                format: super::ImageFormat::Iso,
                license_file: "license.dat".into(),
            },
        )
        .unwrap();
        let xml = std::fs::read_to_string(build.join("disc.xml")).unwrap();
        assert!(xml.contains("image_name=\"epok.iso\""));
        assert!(!xml.contains("cue_sheet="));
        assert!(xml.contains("volume=\"EPOK-SCEE\""));
        assert!(xml.contains("<license file=\""));
        std::fs::remove_dir_all(root).unwrap();
    }
}
