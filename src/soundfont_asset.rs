//! Authoritative SF2/SF3 SoundBank snapshots. The parsed catalog remains derived
//! from the source bytes so an import never publishes one asset per sample.
use crate::{assets, sound_bank};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};
use uuid::Uuid;

pub const PLAYBACK_BLOCKER: &str = "Select this instrument library on a MusicSequence and use Source / PSX Target Preview. A library is cooked per sequence, not as a standalone sound. The imported SF2/SF3 source remains intact.";
pub const REFERENCE_HASH: &str = "cda013d8c370a48ae8dad271e761078d2e77455488dabdedbfbe5fc76a38c682";
pub const REFERENCE_LICENSE: &str = include_str!("../resources/audio/LICENSE-fluidr3mono.txt");

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    pub version: u32,
    pub format: crate::sf2::SourceFormat,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Definition {
    pub fn detected(format: crate::sf2::SourceFormat) -> Self {
        Self {
            version: 1,
            format,
            extra: BTreeMap::new(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported SoundFont source definition version".into());
        }
        Ok(())
    }
}

pub type Candidate = assets::Candidate;

/// A UI-safe source discriminator. Parsing the catalog remains worker-only.
pub fn has_source_header(path: &Path) -> bool {
    let mut header = [0u8; 12];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_ok()
        && crate::sf2::has_header(&header)
}

/// Parse an immutable package snapshot and reject a metadata/source format mismatch.
pub fn decode(package: &assets::Package) -> Result<crate::instrument_ir::LibraryIr, String> {
    if package.meta.kind != assets::Kind::SoundBank {
        return Err("SoundFont source must belong to a SoundBank asset".into());
    }
    let definition = package
        .meta
        .settings
        .sound_bank()?
        .library
        .as_ref()
        .ok_or("SoundBank does not contain a SoundFont source definition")?;
    definition.validate()?;
    let library = crate::sf2::parse(&package.source)?;
    if library.format != definition.format {
        return Err(
            "SoundFont source format changed during import; retry from the current source".into(),
        );
    }
    Ok(library)
}

pub fn inspect(source: &[u8]) -> Result<Vec<(String, String)>, String> {
    let library = crate::sf2::parse(source)?;
    let mut fields = vec![
        ("Instrument library".into(), format!("{:?}: {} presets; {} regions; {} embedded samples",
            library.format, library.presets.len(), library.presets.iter().map(|p| p.regions.len()).sum::<usize>(), library.samples.len())),
        ("Library source".into(), format!("{} bytes; SHA-256 {}", source.len(), assets::hash(source))),
        ("Target cost scope".into(), "Reachable regions are selected per MusicSequence. Source file size is not PSX RAM usage; sample, layer and SFX costs require the conversion report.".into()),
    ];
    for preset in &library.presets {
        fields.push((format!("Library bank {} / program {}", preset.bank, preset.program),
            format!("{} · {} regions; all matching layers retained", preset.name, preset.regions.len())));
    }
    for note in library.diagnostics.iter().chain(&library.blockers).take(32) {
        fields.push((format!("Library report: {}", note.code), note.message.clone()));
    }
    if library.diagnostics.len() + library.blockers.len() > 32 {
        fields.push(("Library report".into(), "Showing the first 32 catalog notes. Conversion reports assess the regions used by the selected song.".into()));
    }
    Ok(fields)
}

/// Prepare, but do not publish, an SF2/SF3 source snapshot. `snapshot` reuses
/// the old authoritative bytes and retains the SoundBank UUID.
pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    mut settings: sound_bank::Settings,
    existing: Option<&assets::Record>,
    snapshot: bool,
) -> Result<Candidate, String> {
    if existing.is_some_and(|record| record.meta.kind != assets::Kind::SoundBank) {
        return Err(
            "Reimport cannot change the asset kind. Import to a new destination instead.".into(),
        );
    }
    let destination = assets::inside(root, destination)?;
    if destination
        .extension()
        .is_none_or(|extension| extension != "epokasset")
    {
        return Err("SoundBank destination must end in .epokasset".into());
    }
    let old = existing
        .map(|record| assets::Package::load(&record.path))
        .transpose()?;
    let previous_definition = old
        .as_ref()
        .and_then(|package| package.meta.settings.sound_bank().ok())
        .and_then(|settings| settings.library.clone());
    if let Some(previous) = old.as_ref().and_then(|package| package.meta.settings.sound_bank().ok()) {
        for (key, value) in &previous.extra { settings.extra.entry(key.clone()).or_insert_with(|| value.clone()); }
        for (key, value) in &previous.envelope_extra { settings.envelope_extra.entry(key.clone()).or_insert_with(|| value.clone()); }
        if settings.provenance.is_empty() { settings.provenance = previous.provenance.clone(); }
    }
    let (bytes, source_path, primary) = if snapshot {
        let old = old
            .as_ref()
            .ok_or("No SoundFont source snapshot to reimport")?;
        if previous_definition.is_none() {
            return Err("Snapshot is not an imported SoundFont SoundBank".into());
        }
        (old.source.clone(), None, old.meta.source.clone())
    } else {
        let path = assets::inside(root, source)?;
        let bytes = assets::read_soundfont_bounded(&path)?;
        (bytes, Some(path), source.replace('\\', "/"))
    };
    let library = crate::sf2::parse(&bytes)?;
    let mut definition = settings
        .library
        .take()
        .or_else(|| previous_definition.clone())
        .unwrap_or_else(|| Definition::detected(library.format.clone()));
    if let Some(previous) = previous_definition {
        for (key, value) in previous.extra {
            definition.extra.entry(key).or_insert(value);
        }
    }
    definition.format = library.format.clone();
    let source_hash = assets::hash(&bytes);
    if source_hash == REFERENCE_HASH {
        // Identification requires the complete checksum, never a name or GM label.
        // Keep the complete redistributable notice in the authoritative asset.
        definition.extra.insert("reference_library".into(), serde_json::json!({
            "id": "fluidr3mono-gm", "version": "2.315", "sha256": REFERENCE_HASH,
            "license": "MIT", "notice": REFERENCE_LICENSE,
        }));
        if settings.provenance.is_empty() {
            settings.provenance = "FluidR3Mono GM 2.315 (Debian 2.315-7), MIT. Full copyright/permission notice is preserved in the library source definition. Reference: https://packages.debian.org/bookworm/fluidr3mono-gm-soundfont".into();
        }
    } else {
        definition.extra.remove("reference_library");
    }
    settings.schema_version = 2;
    settings.library = Some(definition);
    settings.validate()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 2,
            id: existing.map_or_else(Uuid::new_v4, |record| record.meta.id),
            kind: assets::Kind::SoundBank,
            importer_version: 2,
            source: primary,
            source_hash: source_hash.clone(),
            settings: crate::import_settings::Settings::SoundBank(settings),
            extra: old
                .as_ref()
                .map(|package| package.meta.extra.clone())
                .unwrap_or_default(),
        },
        source: bytes,
    };
    decode(&package)?;
    Ok(assets::Candidate {
        destination,
        package,
        expected: existing.map(|record| record.revision.clone()),
        source_path,
        source_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("epok-soundfont-{}", Uuid::new_v4()));
            fs::create_dir_all(root.join("assets")).unwrap();
            fs::write(root.join("assets/library.sf2"), crate::sf2::fixture()).unwrap();
            Self(root)
        }
        fn record(&self) -> assets::Record {
            let path = self.0.join("assets/library.epokasset");
            let package = assets::Package::load(&path).unwrap();
            assets::Record {
                path: path.clone(),
                meta: package.meta,
                revision: assets::hash(&fs::read(path).unwrap()),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixture_33_mib() -> Vec<u8> {
        let mut source = crate::sf2::fixture();
        let smpl = source
            .windows(4)
            .position(|bytes| bytes == b"smpl")
            .unwrap();
        let data_start = smpl + 8;
        let previous = u32::from_le_bytes(source[smpl + 4..smpl + 8].try_into().unwrap()) as usize;
        let sample_bytes = 33 * 1024 * 1024;
        let sdta = source.windows(4).position(|bytes| bytes == b"sdta").unwrap();
        let old_list_size = u32::from_le_bytes(source[sdta - 4..sdta].try_into().unwrap()) as usize;
        source.splice(
            data_start + previous..data_start + previous,
            std::iter::repeat(0).take(sample_bytes - previous),
        );
        source[smpl + 4..smpl + 8].copy_from_slice(&(sample_bytes as u32).to_le_bytes());
        source[sdta - 4..sdta].copy_from_slice(&((old_list_size + sample_bytes - previous) as u32).to_le_bytes());
        let shdr = source
            .windows(4)
            .position(|bytes| bytes == b"shdr")
            .unwrap();
        let sample_count = (sample_bytes / 2) as u32;
        source[shdr + 8 + 24..shdr + 8 + 28].copy_from_slice(&sample_count.to_le_bytes());
        source[shdr + 8 + 32..shdr + 8 + 36].copy_from_slice(&sample_count.to_le_bytes());
        let riff_size = (source.len() - 8) as u32;
        source[4..8].copy_from_slice(&riff_size.to_le_bytes());
        assert!(crate::sf2::parse(&source).is_ok());
        source
    }

    #[test]
    fn reimport_preserves_authoritative_source_uuid_and_unknown_fields() {
        let fixture = Fixture::new();
        let mut settings = sound_bank::Settings::default();
        settings.extra.insert("futureBankFlag".into(), serde_json::json!(42));
        let mut definition = Definition::detected(crate::sf2::SourceFormat::Sf2Pcm16);
        definition
            .extra
            .insert("futureSourceFlag".into(), serde_json::json!(true));
        settings.library = Some(definition);
        let original = fs::read(fixture.0.join("assets/library.sf2")).unwrap();
        let mut first = prepare(
            &fixture.0,
            "assets/library.sf2",
            "assets/library.epokasset",
            settings,
            None,
            false,
        )
        .unwrap();
        first
            .package
            .meta
            .extra
            .insert("futurePackageFlag".into(), serde_json::json!("kept"));
        let id = first.package.meta.id;
        assets::commit(first).unwrap();

        let package = assets::Package::load(&fixture.0.join("assets/library.epokasset")).unwrap();
        let settings = package.meta.settings.sound_bank().unwrap();
        assert_eq!(package.source, original);
        assert_eq!(package.meta.id, id);
        assert!(settings.dependencies().is_empty());
        assert!(settings.programs.is_empty());
        assert!(
            settings
                .validate_playback()
                .unwrap_err()
                .contains("not available yet")
        );

        let record = fixture.record();
        let stale = prepare(
            &fixture.0,
            "assets/library.sf2",
            "assets/library.epokasset",
            sound_bank::Settings::default(),
            Some(&record),
            false,
        )
        .unwrap();
        let mut changed_source = original.clone();
        let smpl = changed_source.windows(4).position(|bytes| bytes == b"smpl").unwrap();
        changed_source[smpl + 8] ^= 1;
        fs::write(fixture.0.join("assets/library.sf2"), changed_source).unwrap();
        assert!(
            assets::commit(stale)
                .unwrap_err()
                .contains("Source changed during import")
        );
        assert_eq!(
            assets::Package::load(&record.path).unwrap().source,
            original
        );

        fs::write(fixture.0.join("assets/library.sf2"), &original).unwrap();
        let reimport = prepare(
            &fixture.0,
            "assets/library.sf2",
            "assets/library.epokasset",
            sound_bank::Settings::default(),
            Some(&record),
            false,
        )
        .unwrap();
        assets::commit(reimport).unwrap();
        let package = assets::Package::load(&record.path).unwrap();
        assert_eq!(package.meta.id, id);
        assert_eq!(package.meta.settings.sound_bank().unwrap().extra["futureBankFlag"], serde_json::json!(42));
        assert_eq!(
            package.meta.extra.get("futurePackageFlag"),
            Some(&serde_json::json!("kept"))
        );
        assert_eq!(
            package
                .meta
                .settings
                .sound_bank()
                .unwrap()
                .library
                .as_ref()
                .unwrap()
                .extra
                .get("futureSourceFlag"),
            Some(&serde_json::json!(true))
        );

        let record = fixture.record();
        let snapshot = prepare(
            &fixture.0,
            "assets/missing.sf2",
            "assets/library.epokasset",
            sound_bank::Settings::default(),
            Some(&record),
            true,
        )
        .unwrap();
        assert!(snapshot.source_path.is_none());
        assert_eq!(snapshot.package.source, original);

        let record = fixture.record();
        let stale_package = prepare(
            &fixture.0,
            "assets/library.sf2",
            "assets/library.epokasset",
            sound_bank::Settings::default(),
            Some(&record),
            false,
        )
        .unwrap();
        assets::make_independent(&record).unwrap();
        assert!(
            assets::commit(stale_package)
                .unwrap_err()
                .contains("Asset changed during the operation")
        );
    }

    #[test]
    fn large_soundfont_is_loadable_and_relinks_without_audio_clip_expansion() {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("assets/library.sf2"), fixture_33_mib()).unwrap();
        let candidate = prepare(
            &fixture.0,
            "assets/library.sf2",
            "assets/library.epokasset",
            sound_bank::Settings::default(),
            None,
            false,
        )
        .unwrap();
        assets::commit(candidate).unwrap();
        let package_path = fixture.0.join("assets/library.epokasset");
        assert!(
            assets::read_bounded(&fixture.0.join("assets/library.sf2"))
                .unwrap()
                .len()
                > 32 * 1024 * 1024
        );
        assert!(assets::read_bounded(&package_path).unwrap().len() > 32 * 1024 * 1024);
        assert!(assets::Package::load(&package_path).unwrap().source.len() > 32 * 1024 * 1024);

        let record = fixture.record();
        let snapshot = prepare(
            &fixture.0,
            "assets/missing.sf2",
            "assets/library.epokasset",
            sound_bank::Settings::default(),
            Some(&record),
            true,
        )
        .unwrap();
        assert!(snapshot.source_path.is_none());
        drop(snapshot);

        assets::move_asset(&fixture.0, &record, "assets/moved.epokasset").unwrap();
        let mut cache = assets::ScanCache::default();
        let index = assets::scan(&fixture.0, &mut cache);
        assert!(
            index
                .usable()
                .all(|record| record.meta.kind != assets::Kind::AudioClip)
        );
        let record = index
            .usable()
            .find(|record| record.path.ends_with("moved.epokasset"))
            .unwrap()
            .clone();
        fs::rename(
            fixture.0.join("assets/library.sf2"),
            fixture.0.join("assets/renamed.sf2"),
        )
        .unwrap();
        let index = assets::scan(&fixture.0, &mut cache);
        let mut relinks = assets::source_relinks(&fixture.0, &index);
        assert_eq!(relinks.len(), 1);
        assets::commit(relinks.pop().unwrap()).unwrap();
        let package = assets::Package::load(&record.path).unwrap();
        assert_eq!(package.meta.source, "assets/renamed.sf2");
    }
}
