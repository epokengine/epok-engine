//! Source-preserving Sony bank adapter. Resolved portable banks and imported
//! banks share inspection; unresolved Sony semantics never reach a mixer/cooker.
use crate::{assets, sound_bank, vab_import};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const PLAYBACK_BLOCKER: &str = "Sony SoundBank playback unavailable: original sample Hz/tuning, native ADSR, staged gain/pan and channel mapping need verified conversion. All tones/layers remain in the source IR. Assign a resolved portable SoundBank; Ignore unsupported MIDI events cannot bypass this bank requirement.";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub path: String,
    pub offset: usize,
    pub bytes: usize,
    pub sha256: String,
    #[serde(default, flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    pub version: u32,
    pub profile: String,
    pub parts: Vec<Part>,
    #[serde(default, flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            version: 1,
            profile: "sony-vab-v7".into(),
            parts: vec![],
            extra: Default::default(),
        }
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.profile != "sony-vab-v7" || self.parts.len() > 2 {
            return Err("Unsupported imported SoundBank definition/profile; expected Sony VAB v7 and one source or an explicit VH/VB pair".into());
        }
        let mut end = 0usize;
        for part in &self.parts {
            assets::validate_source_path(&part.path)?;
            if part.offset != end
                || part.bytes == 0
                || part.sha256.len() != 64
                || !part.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("Invalid SoundBank source-part span/hash".into());
            }
            end = end
                .checked_add(part.bytes)
                .ok_or("SoundBank source span overflow")?;
            if end > vab_import::MAX_SOURCE_BYTES {
                return Err("SoundBank source exceeds 4 MiB".into());
            }
        }
        if self.parts.len() == 2 && self.parts[0].path == self.parts[1].path {
            return Err("VH and VB must be separate explicitly selected files".into());
        }
        Ok(())
    }
}

/// The same logical view is available for resolved native zones and unresolved
/// source tones. None means unresolved, never a fabricated default.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ToneView {
    pub program: u8,
    pub drum_key: Option<u8>,
    pub keys: [u8; 2],
    pub velocity: Option<[u8; 2]>,
    pub root_key: u8,
    pub fine_tune_cents: Option<f32>,
    pub gain: Option<f32>,
    pub pan: Option<f32>,
    pub envelope: Option<sound_bank::Envelope>,
    pub sample_asset: Option<Uuid>,
    pub source_sample_id: Option<u16>,
}
pub enum SourceIr {
    Native(sound_bank::Settings),
    Sony(vab_import::BankImportIr),
}
impl SourceIr {
    pub fn tones(&self) -> Vec<ToneView> {
        match self {
            Self::Native(s) => s
                .programs
                .iter()
                .flat_map(|p| {
                    p.zones.iter().map(move |z| ToneView {
                        program: p.program,
                        drum_key: p.drum_key,
                        keys: z.key_range,
                        velocity: Some(z.velocity_range),
                        root_key: z.root_key,
                        fine_tune_cents: Some(z.fine_tune_cents),
                        gain: Some(z.gain),
                        pan: Some(z.pan),
                        envelope: Some(z.envelope.clone()),
                        sample_asset: Some(z.sample),
                        source_sample_id: None,
                    })
                })
                .collect(),
            Self::Sony(s) => s
                .programs
                .iter()
                .flat_map(|p| {
                    p.tones.iter().map(move |t| ToneView {
                        program: p.id,
                        drum_key: None,
                        keys: t.keys,
                        velocity: None,
                        root_key: t.root_key,
                        fine_tune_cents: None,
                        gain: None,
                        pan: None,
                        envelope: None,
                        sample_asset: None,
                        source_sample_id: Some(t.sample_id),
                    })
                })
                .collect(),
        }
    }
    pub fn resolve(&self) -> Result<&sound_bank::Settings, String> {
        match self {
            Self::Native(s) => {
                s.validate()?;
                s.validate_playback()?;
                Ok(s)
            }
            Self::Sony(s) => {
                let issues = vab_import::assess_current_sound_bank(s);
                Err(format!(
                    "{PLAYBACK_BLOCKER}\n{}",
                    issues
                        .iter()
                        .map(|d| format!(
                            "{} at part {} offset 0x{:x}: {}",
                            d.code, d.at.part, d.at.offset, d.reason
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
        }
    }
}

pub fn decode(package: &assets::Package) -> Result<SourceIr, String> {
    let settings = package.meta.settings.sound_bank()?;
    let Some(definition) = &settings.imported else {
        return Ok(SourceIr::Native(settings.clone()));
    };
    definition.validate()?;
    if definition.parts.is_empty() {
        return Err("Imported SoundBank has no source-parts manifest".into());
    }
    let mut parts = Vec::new();
    for part in &definition.parts {
        let data = package
            .source
            .get(part.offset..part.offset + part.bytes)
            .ok_or("Truncated SoundBank source part")?;
        if assets::hash(data) != part.sha256 {
            return Err(format!(
                "SoundBank source part {} checksum mismatch",
                part.path
            ));
        }
        parts.push(vab_import::Input {
            bytes: data,
            label: &part.path,
            rights: &settings.provenance,
        });
    }
    let last = definition.parts.last().unwrap();
    if last.offset + last.bytes != package.source.len() {
        return Err("SoundBank parts do not cover source exactly".into());
    }
    let first = parts.remove(0);
    vab_import::parse(first, parts.pop())
        .map(SourceIr::Sony)
        .map_err(|e| e.to_string())
}

pub fn inspect(package: &assets::Package) -> Result<Vec<(String, String)>, String> {
    let ir = decode(package)?;
    let mut fields = vec![("Logical source tones".into(), ir.tones().len().to_string())];
    if let SourceIr::Sony(bank) = &ir {
        fields.extend([
            ("Detected bank profile".into(), bank.profile.clone()),
            ("Imported source".into(),format!("{} programs, {} tones, {} samples; all source layers retained",bank.header.program_count,bank.header.tone_count,bank.header.sample_count)),
            ("Playback compatibility".into(),PLAYBACK_BLOCKER.into()),
            ("Decoded source quality".into(),"Only PSX ADPCM source is available. First-pass PCM is decoded for inspection; future re-encoding adds loss. Original Hz is unknown; no invented-rate audition.".into()),
        ]);
        for source in &bank.sources {
            fields.push((
                format!("Source {}", source.label),
                format!("{} bytes; SHA-256 {}", source.bytes.len(), source.sha256),
            ));
        }
        for p in &bank.programs {
            fields.push((
                format!("Program {}", p.id),
                format!(
                    "{} tones; overlapping pairs {:?}; gain {}, pan {}, priority {}, mode {}",
                    p.tones.len(),
                    p.overlapping_tone_pairs,
                    p.gain,
                    p.pan,
                    p.priority,
                    p.mode
                ),
            ));
            for t in &p.tones {
                fields.push((format!("Program {}, tone {}",p.id,t.slot),format!("sample {}; keys {:?}; root {}; raw tune {}; gain/pan {}/{}; ADSR {:04x}/{:04x}; bend {:?}; mode {}; priority {}; vibrato {:?}; portamento {:?}; source offset 0x{:x}",t.sample_id,t.keys,t.root_key,t.tuning_raw,t.gain,t.pan,t.adsr_words[0],t.adsr_words[1],t.bend_down_up,t.mode,t.priority,t.vibrato,t.portamento,t.at.offset)));
            }
        }
        for s in &bank.samples {
            fields.push((
                format!("Sample {}", s.id),
                format!(
                    "{} encoded bytes; {} first-pass frames; {:?}; loop {:?}; PCM SHA-256 {}",
                    s.encoded.length,
                    s.decoded.pcm.len(),
                    s.decoded.termination,
                    s.decoded.loop_region,
                    assets::hash(
                        &s.decoded
                            .pcm
                            .iter()
                            .flat_map(|v| v.to_le_bytes())
                            .collect::<Vec<_>>()
                    )
                ),
            ));
        }
        for d in vab_import::assess_current_sound_bank(bank) {
            fields.push((
                format!("Compatibility: {}", d.code),
                format!(
                    "part {}, offset 0x{:x}: {}",
                    d.at.part, d.at.offset, d.reason
                ),
            ));
        }
    }
    Ok(fields)
}

pub struct Candidate {
    inner: assets::Candidate,
    observed: Vec<(PathBuf, String)>,
}
pub fn prepare(
    root: &Path,
    source: &str,
    companion: Option<&str>,
    destination: &str,
    mut settings: sound_bank::Settings,
    existing: Option<&assets::Record>,
    snapshot: bool,
) -> Result<Candidate, String> {
    if existing.is_some_and(|r| r.meta.kind != assets::Kind::SoundBank) {
        return Err("Reimport cannot change asset kind".into());
    }
    let path = assets::inside(root, destination)?;
    if path.extension().is_none_or(|e| e != "epokasset") {
        return Err("SoundBank destination must end in .epokasset".into());
    }
    let old = existing
        .map(|r| assets::Package::load(&r.path))
        .transpose()?;
    let mut observed = Vec::new();
    let (bytes, primary, definition) = if snapshot {
        let old = old.as_ref().ok_or("No SoundBank snapshot to reimport")?;
        let definition = old
            .meta
            .settings
            .sound_bank()?
            .imported
            .clone()
            .ok_or("Snapshot is not an imported Sony SoundBank")?;
        (old.source.clone(), old.meta.source.clone(), definition)
    } else {
        let mut paths = vec![source.to_owned()];
        let companion = match companion {
            Some(path) => (!path.is_empty()).then(|| path.to_owned()),
            None => settings
                .imported
                .as_ref()
                .and_then(|d| d.parts.get(1))
                .map(|p| p.path.clone()),
        };
        if let Some(companion) = companion {
            paths.push(companion);
        }
        let mut definition = settings.imported.clone().unwrap_or_default();
        let previous_parts = std::mem::take(&mut definition.parts);
        let mut bytes = Vec::new();
        for name in paths {
            let path = assets::inside(root, &name)?;
            let data = assets::read_bounded(&path)?;
            let hash = assets::hash(&data);
            definition.parts.push(Part {
                path: name.replace('\\', "/"),
                offset: bytes.len(),
                bytes: data.len(),
                sha256: hash.clone(),
                extra: previous_parts
                    .get(definition.parts.len())
                    .map(|p| p.extra.clone())
                    .unwrap_or_default(),
            });
            observed.push((path, hash));
            if bytes.len() + data.len() > vab_import::MAX_SOURCE_BYTES {
                return Err("SoundBank source exceeds 4 MiB".into());
            }
            bytes.extend(data);
        }
        (bytes, source.replace('\\', "/"), definition)
    };
    settings.imported = Some(definition);
    settings.validate()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 2,
            id: existing.map_or_else(Uuid::new_v4, |r| r.meta.id),
            kind: assets::Kind::SoundBank,
            importer_version: 2,
            source: primary,
            source_hash: assets::hash(&bytes),
            settings: crate::import_settings::Settings::SoundBank(settings),
            extra: old
                .as_ref()
                .map(|p| p.meta.extra.clone())
                .unwrap_or_default(),
        },
        source: bytes,
    };
    decode(&package)?;
    package.bytes()?;
    Ok(Candidate {
        inner: assets::Candidate {
            destination: path,
            source_hash: package.meta.source_hash.clone(),
            package,
            expected: existing.map(|r| r.revision.clone()),
            source_path: None,
        },
        observed,
    })
}
pub fn commit(candidate: Candidate) -> Result<Uuid, String> {
    for (path, hash) in &candidate.observed {
        if assets::hash(&assets::read_bounded(path)?) != *hash {
            return Err(
                "SoundBank source pair changed during import; previous asset is intact. Retry."
                    .into(),
            );
        }
    }
    assets::commit(candidate.inner)
}

pub fn parts(record: &assets::Record) -> Option<&[Part]> {
    record
        .meta
        .settings
        .sound_bank()
        .ok()?
        .imported
        .as_ref()
        .map(|d| d.parts.as_slice())
}
pub fn source_candidate(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| matches!(e.to_ascii_lowercase().to_str(), Some("vab" | "vh" | "vb")))
}
pub fn has_header(bytes: &[u8]) -> bool {
    bytes.starts_with(b"pBAV")
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub fn fixture() -> Vec<u8> {
        let mut b = vec![0u8; 0xa20 + 512 + 16];
        b[..4].copy_from_slice(b"pBAV");
        b[4..8].copy_from_slice(&7u32.to_le_bytes());
        let size = b.len() as u32;
        b[12..16].copy_from_slice(&size.to_le_bytes());
        b[18] = 1;
        b[20] = 2;
        b[22] = 1;
        b[24] = 127;
        b[25] = 64;
        b[32] = 2;
        b[33] = 127;
        b[36] = 64;
        for t in [0x820, 0x840] {
            b[t + 2] = 127;
            b[t + 3] = 64;
            b[t + 4] = 60;
            b[t + 7] = 127;
            b[t + 12] = 2;
            b[t + 13] = 2;
            b[t + 22] = 1;
        }
        b[0xa22] = 2;
        b[0xc20] = 12;
        b[0xc21] = 1;
        b[0xc22] = 0x71;
        b
    }
    #[test]
    fn sony_bank_pair_is_atomic_preserves_layers_and_never_becomes_empty_playback() {
        let root = crate::workspace::tests::temp("sony-bank-pair");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let bytes = fixture();
        std::fs::write(root.join("assets/sound.vh"), &bytes[..0xc20]).unwrap();
        std::fs::write(root.join("assets/sound.vb"), &bytes[0xc20..]).unwrap();
        let settings = sound_bank::Settings {
            provenance: "Original generated Epok test, MIT".into(),
            ..Default::default()
        };
        let id = commit(
            prepare(
                &root,
                "assets/sound.vh",
                Some("assets/sound.vb"),
                "assets/bank.epokasset",
                settings.clone(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap();
        let package = assets::Package::load(&record.path).unwrap();
        assert_eq!(package.source, bytes);
        let ir = decode(&package).unwrap();
        assert_eq!(ir.tones().len(), 2);
        assert!(ir.tones().iter().all(|t| t.envelope.is_none()));
        assert!(ir.resolve().unwrap_err().contains("ADSR"));
        assert!(
            crate::psx_sequence::bank(&root, record, &index)
                .err()
                .unwrap()
                .contains("Sony")
        );
        assert!(
            inspect(&package)
                .unwrap()
                .iter()
                .any(|(k, _)| k.contains("psx-layers"))
        );
        let original = std::fs::read(&record.path).unwrap();
        let pending = prepare(
            &root,
            "assets/sound.vh",
            Some("assets/sound.vb"),
            "assets/bank.epokasset",
            settings.clone(),
            Some(record),
            false,
        )
        .unwrap();
        std::fs::write(root.join("assets/sound.vb"), [0; 16]).unwrap();
        assert!(commit(pending).is_err());
        assert_eq!(std::fs::read(&record.path).unwrap(), original);
        std::fs::remove_file(root.join("assets/sound.vh")).unwrap();
        std::fs::remove_file(root.join("assets/sound.vb")).unwrap();
        assert_eq!(
            commit(
                prepare(
                    &root,
                    "",
                    None,
                    "assets/bank.epokasset",
                    package.meta.settings.sound_bank().unwrap().clone(),
                    Some(record),
                    true
                )
                .unwrap()
            )
            .unwrap(),
            id
        );
        let reloaded = assets::Package::load(&record.path).unwrap();
        assert_eq!(decode(&reloaded).unwrap().tones(), ir.tones());
        assert_eq!(reloaded.source, bytes);
        std::fs::write(root.join("assets/combined.vab"), &bytes).unwrap();
        let fresh = assets::scan(&root, &mut Default::default());
        assert_eq!(
            commit(
                prepare(
                    &root,
                    "assets/combined.vab",
                    Some(""),
                    "assets/bank.epokasset",
                    reloaded.meta.settings.sound_bank().unwrap().clone(),
                    Some(fresh.resolve(id).unwrap()),
                    false
                )
                .unwrap()
            )
            .unwrap(),
            id
        );
        let combined = assets::Package::load(&record.path).unwrap();
        assert_eq!(combined.source, bytes);
        assert_eq!(
            combined
                .meta
                .settings
                .sound_bank()
                .unwrap()
                .imported
                .as_ref()
                .unwrap()
                .parts
                .len(),
            1
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
