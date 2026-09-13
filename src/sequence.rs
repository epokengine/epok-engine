//! Portable sequence authoring. Complete original files stay in asset snapshots.
use crate::audio_import::{AudioRole, LoadMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use uuid::Uuid;
pub use crate::sequence_compat::Profile as SourceProfile;

/// Persisted explicit choice. Sony supplies song IDs; converted SEQ/SEP supplies only an
/// ordinal, guarded by the record hash so changed/reordered songs need reselection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSelection {
    pub schema_version: u32,
    pub profile: SourceProfile,
    pub song_id: Option<u16>,
    pub song_index: Option<u16>,
    pub record_hash: Option<String>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl SourceSelection {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Unsupported sequence source selection version".into());
        }
        let valid = match self.profile {
            SourceProfile::SonySeqV1 => self.song_id == Some(0) && self.song_index.is_none() && self.record_hash.is_none(),
            SourceProfile::SonySepV0 => self.song_id.is_some() && self.song_index.is_none() && self.record_hash.is_none(),
            SourceProfile::ConvertedSeqLe32V1 => self.song_id.is_none() && self.song_index.is_some_and(|i| i < 256)
                && self.record_hash.as_ref().is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())),
        };
        if valid { Ok(()) } else { Err("Invalid source selection: Sony requires song ID; converted SEQ/SEP requires ordinal and lowercase SHA-256 record hash".into()) }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceSong {
    pub id: u16,
    pub ordinal: u16,
    pub offset: usize,
    pub length: usize,
    pub record_hash: String,
    pub ppqn: u16,
    pub duration_micros: u64,
    pub events: usize,
    pub playback_blockers: Vec<String>,
    pub source_loop: Option<crate::sequence_compat::SourceLoop>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SourceCatalog {
    /// None is MIDI; Some identifies a fully structurally verified compatibility format.
    pub profile: Option<SourceProfile>,
    pub songs: Vec<SourceSong>,
}

/// Analysis only. Never silently selects a SEP song, imports or cooks an asset.
pub fn catalog_source(bytes: &[u8], explicit: Option<SourceProfile>) -> Result<SourceCatalog, String> {
    if bytes.starts_with(b"MThd") {
        if explicit.is_some() { return Err("MIDI source does not match the selected compatibility profile".into()); }
        let ir = crate::midi::parse(bytes)?;
        return Ok(SourceCatalog { profile: None, songs: vec![SourceSong { id: 0, ordinal: 0, offset: 0,
            length: bytes.len(), record_hash: crate::assets::hash(bytes), ppqn: ir.ppqn,
            duration_micros: ir.duration_micros, events: ir.events.len(), playback_blockers: vec![], source_loop: None }] });
    }
    let profile = match explicit { Some(p) => p, None => crate::sequence_compat::detect(bytes)? };
    let container = crate::sequence_compat::parse(bytes, profile)?;
    Ok(SourceCatalog { profile: Some(profile), songs: container.entries.into_iter().enumerate().map(|(ordinal, e)| SourceSong {
        id: e.id, ordinal: ordinal as u16, offset: e.offset, length: e.length,
        record_hash: crate::assets::hash(&bytes[e.offset..e.offset + e.length]), ppqn: e.ir.ppqn,
        duration_micros: e.ir.duration_micros, events: e.ir.events.len(),
        playback_blockers: e.ir.playback_blockers.iter().map(|d| d.message.clone()).collect(), source_loop: e.source_loop,
    }).collect() })
}

/// Build a choice only after the caller explicitly selects a catalog song.
pub fn select_source(bytes: &[u8], profile: SourceProfile, song_id: Option<u16>, song_index: Option<u16>) -> Result<SourceSelection, String> {
    let catalog = catalog_source(bytes, Some(profile))?;
    let song = match profile {
        SourceProfile::ConvertedSeqLe32V1 if song_id.is_none() => catalog.songs.iter().find(|s| Some(s.ordinal) == song_index),
        SourceProfile::SonySeqV1 | SourceProfile::SonySepV0 if song_index.is_none() => catalog.songs.iter().find(|s| Some(s.id) == song_id),
        _ => None,
    }.ok_or("Choose an existing Sony song ID or converted song ordinal from the source catalog")?;
    let selection = SourceSelection { schema_version: 1, profile, song_id, song_index,
        record_hash: (profile == SourceProfile::ConvertedSeqLe32V1).then(|| song.record_hash.clone()), extra: Default::default() };
    selection.validate()?;
    Ok(selection)
}

/// Shared inspection/preview/cook decode. Returns source fidelity diagnostics;
/// `Settings::validate_playback` applies hard blockers before playback/cooking.
pub fn decode_source(bytes: &[u8], settings: &Settings) -> Result<crate::sequence_ir::SequenceIr, String> {
    settings.validate()?;
    let Some(selection) = &settings.source_selection else {
        if bytes.starts_with(b"MThd") { return crate::midi::parse_with_profile(bytes, settings.midi_profile); }
        let catalog = catalog_source(bytes, None)?;
        return Err(format!("{} contains {} song(s). Select the source profile and song explicitly before import or audition.",
            catalog.profile.map_or("MIDI", |p| p.id()), catalog.songs.len()));
    };
    let container = crate::sequence_compat::parse(bytes, selection.profile)?;
    let selected = container.entries.into_iter().enumerate().find(|(ordinal, e)| match selection.profile {
        SourceProfile::ConvertedSeqLe32V1 => Some(*ordinal as u16) == selection.song_index,
        _ => Some(e.id) == selection.song_id,
    }).ok_or("Selected sequence song is absent; reselect from the current source catalog. The previous asset is unchanged.")?.1;
    if selection.profile == SourceProfile::ConvertedSeqLe32V1
        && selection.record_hash.as_deref() != Some(crate::assets::hash(&bytes[selected.offset..selected.offset + selected.length]).as_str()) {
        return Err("Selected converted record changed or moved. Explicitly reselect its ordinal from the current catalog; the previous asset is unchanged.".into());
    }
    Ok(selected.ir)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    #[default]
    Off,
    Whole,
    Markers,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub role: AudioRole,
    pub load_mode: LoadMode,
    pub sound_bank: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instrument_mappings: Vec<crate::instrument_selection::Mapping>,
    pub loop_mode: LoopMode,
    /// None selects the documented initial ceiling of 16. Never reduced to fit a bank.
    pub voice_limit: Option<u16>,
    pub ignore_unsupported: bool,
    /// Missing on old packages: retain their original interpretation until explicitly upgraded.
    #[serde(default = "legacy_midi_profile", skip_serializing_if = "is_legacy_midi_profile")]
    pub midi_profile: crate::midi::MidiProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_selection: Option<SourceSelection>,
    pub target_overrides: BTreeMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    #[serde(skip)]
    pub envelope_extra: BTreeMap<String, serde_json::Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            role: AudioRole::Music,
            load_mode: LoadMode::Resident,
            sound_bank: None,
            instrument_mappings: Vec::new(),
            loop_mode: LoopMode::Off,
            voice_limit: None,
            ignore_unsupported: false,
            midi_profile: crate::midi::MidiProfile::MusicalV2,
            source_selection: None,
            target_overrides: BTreeMap::new(),
            extra: BTreeMap::new(),
            envelope_extra: BTreeMap::new(),
        }
    }
}
fn legacy_midi_profile() -> crate::midi::MidiProfile {
    crate::midi::MidiProfile::LegacyV1
}
fn is_legacy_midi_profile(profile: &crate::midi::MidiProfile) -> bool {
    *profile == crate::midi::MidiProfile::LegacyV1
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Unsupported MusicSequence settings version".into());
        }
        if let Some(selection) = &self.source_selection { selection.validate()?; }
        crate::instrument_selection::validate_mappings(&self.instrument_mappings)?;
        if self.sound_bank.is_some_and(|id| id.is_nil()) {
            return Err("SoundBank UUID cannot be nil".into());
        }
        if self
            .voice_limit
            .is_some_and(|limit| !(1..=128).contains(&limit))
        {
            return Err("Music voice limit must be 1–128 or Auto".into());
        }
        if self
            .target_overrides
            .values()
            .any(|value| !value.is_object())
        {
            return Err("Sequence target overrides must be namespaced objects".into());
        }
        crate::psx_music_settings::Recipe::from_settings(self)?;
        Ok(())
    }
    pub fn voices(&self) -> u16 {
        self.voice_limit.unwrap_or(16)
    }
    pub fn validate_playback(&self, ir: &crate::sequence_ir::SequenceIr) -> Result<(), String> {
        self.validate()?;
        if ir.events.iter().any(|event| matches!(event.kind, crate::sequence_ir::EventKind::Control { controller: 91, value: 1..=127, .. }))
            && crate::psx_music_settings::Recipe::from_settings(self)?.effects != crate::psx_music_settings::Effects::Room
        {
            return Err("Nonzero MIDI CC91 requires the explicit PSX Room reverb policy and an instrument library; Dry cannot discard the requested effect".into());
        }
        if let Some(first) = ir.playback_blockers.first() {
            if ir.source_profile.as_deref() == Some("midi-musical-v2") {
                return Err(format!("MIDI contains {} unsupported events in Musical v2: {} (track {}, tick {}). Review the report and re-export the MIDI with these operations resolved. Ignore unsupported events is a Legacy v1 exception and cannot bypass this profile.",
                    ir.playback_blockers.len(), first.message, first.track + 1, first.tick));
            }
            return Err(format!("{} playback blocked by {} unresolved source requirements: {}. Ignore unsupported MIDI cannot bypass this incompatibility.",
                ir.source_profile.as_deref().unwrap_or("Sequence"), ir.playback_blockers.len(), first.message));
        }
        if self.load_mode == LoadMode::Stream {
            return Err("Sequence events must be Resident in the initial profile. SoundBank residency is separate.".into());
        }
        if self.loop_mode == LoopMode::Markers && ir.loop_region.is_none() {
            return Err("Add loop_start and loop_end MIDI markers or choose Whole/Off".into());
        }
        if !self.ignore_unsupported {
            let unsupported = ir
                .diagnostics
                .iter()
                .filter(|d| d.unsupported)
                .collect::<Vec<_>>();
            if let Some(first) = unsupported.first() {
                return Err(format!(
                    "MIDI contains {} unsupported events: {} (track {}, tick {}). Review the report and explicitly enable Ignore unsupported events, or re-export the MIDI.",
                    unsupported.len(),
                    first.message,
                    first.track + 1,
                    first.tick
                ));
            }
        }
        Ok(())
    }
}

pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    settings: Settings,
    existing: Option<&crate::assets::Record>,
    snapshot: bool,
) -> Result<crate::assets::Candidate, String> {
    crate::assets::prepare_portable(
        root,
        source,
        destination,
        crate::import_settings::Settings::MusicSequence(settings),
        existing,
        snapshot,
    )
}

pub fn resolve_bank<'a>(
    root: &Path,
    settings: &Settings,
    index: &'a crate::assets::Index,
) -> Result<&'a crate::assets::Record, String> {
    let id = settings.sound_bank.or(crate::workspace::optional_manifest(root)?.and_then(|m| m.default_sound_bank))
        .ok_or("MIDI contains notes, not instruments. Assign a SoundBank in the MusicSequence Inspector or select a Project Default SoundBank.")?;
    let bank = index.resolve(id)?;
    if bank.meta.kind != crate::assets::Kind::SoundBank {
        return Err(format!(
            "Asset {id} is not a SoundBank. Select a SoundBank asset."
        ));
    }
    Ok(bank)
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;
    fn score(key: u8) -> Vec<u8> { vec![0, 0x90, key, 100, 96, 0x80, key, 17, 0, 0xff, 0x2f, 0] }
    fn seq(events: &[u8]) -> Vec<u8> {
        let mut bytes = b"pQES\0\0\0\x01\0\x60\x07\xa1\x20\x04\x02".to_vec(); bytes.extend(events); bytes
    }
    fn sep() -> Vec<u8> {
        let mut bytes = b"pQES\0\0".to_vec();
        for (id, key) in [(4_u16, 60), (9_u16, 72)] {
            let events = score(key); bytes.extend(id.to_be_bytes()); bytes.extend([0, 96, 7, 0xa1, 0x20, 4, 2]);
            bytes.extend((events.len() as u32).to_be_bytes()); bytes.extend(events);
        }
        bytes
    }
    fn converted_seq(key: u8) -> Vec<u8> {
        let events = [0x90, key, 100, 96, 0x80, key, 17, 0, 0xff, 0x2f, 0];
        let size = (12 + events.len()).next_multiple_of(4);
        let mut b = (size as u32).to_le_bytes().to_vec(); b.extend(500000_u32.to_le_bytes());
        b.extend([96, 0, 4, 2]); b.extend(events); b.resize(size, 0); b
    }
    #[test]
    fn compatibility_song_selection_is_explicit_and_never_first_entry_fallback() {
        let bytes = sep(); let c = catalog_source(&bytes, None).unwrap();
        assert_eq!(c.profile, Some(SourceProfile::SonySepV0)); assert_eq!(c.songs.iter().map(|s|s.id).collect::<Vec<_>>(), [4, 9]);
        assert!(decode_source(&bytes, &Settings::default()).unwrap_err().contains("explicitly"));
        let mut settings = Settings { source_selection: Some(select_source(&bytes, SourceProfile::SonySepV0, Some(9), None).unwrap()), ..Default::default() };
        let ir = decode_source(&bytes, &settings).unwrap(); settings.validate_playback(&ir).unwrap();
        assert_eq!(ir.source_events[0].data, [72, 100]);
        settings.source_selection.as_mut().unwrap().song_id = Some(33);
        assert!(decode_source(&bytes, &settings).unwrap_err().contains("absent"));
        assert!(select_source(&bytes, SourceProfile::SonySepV0, None, Some(0)).is_err());
    }
    #[test]
    fn converted_reimport_rejects_changed_or_reordered_selection_until_reselected() {
        let mut bytes = converted_seq(60); bytes.extend(converted_seq(72));
        let mut settings = Settings { source_selection: Some(select_source(&bytes, SourceProfile::ConvertedSeqLe32V1, None, Some(1)).unwrap()), ..Default::default() };
        assert_eq!(decode_source(&bytes, &settings).unwrap().source_events[0].data[0], 72);
        let mut changed = converted_seq(72); changed.extend(converted_seq(60));
        assert!(decode_source(&changed, &settings).unwrap_err().contains("reselect"));
        settings.source_selection = Some(select_source(&changed, SourceProfile::ConvertedSeqLe32V1, None, Some(0)).unwrap());
        assert_eq!(decode_source(&changed, &settings).unwrap().source_events[0].data[0], 72);
        assert!(decode_source(&changed[..changed.len()-1], &settings).is_err(), "unselected songs must also be valid");
    }
    #[test]
    fn complete_source_ledger_roundtrips_and_hard_blockers_ignore_no_midi_toggle() {
        let bytes = seq(&[0, 0xb0, 0, 3, 0, 0xa0, 60, 64, 0, 0x80, 60, 31, 0, 0xff, 0x51, 7, 0xa1, 0x20, 0, 0x2f, 0]);
        let mut settings = Settings { source_selection: Some(select_source(&bytes, SourceProfile::SonySeqV1, Some(0), None).unwrap()), ignore_unsupported: true, ..Default::default() };
        let ir = decode_source(&bytes, &settings).unwrap();
        assert_eq!(ir.source_events.len(), 5);
        assert_eq!(ir.source_events[0].data, [0, 3]); assert_eq!(ir.source_events[2].data, [60, 31]);
        assert_eq!(ir.source_events[3].data, [0x51, 7, 0xa1, 0x20]);
        assert!(!ir.source_events[4].explicit_status); assert_eq!(ir.source_events[4].status, 0xff);
        let encoded = serde_json::to_vec(&ir.source_events).unwrap();
        let restored: Vec<crate::sequence_ir::SourceEvent> = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(restored, ir.source_events);
        assert!(settings.validate_playback(&ir).unwrap_err().contains("cannot bypass"));
        settings.ignore_unsupported = false; assert!(settings.validate_playback(&ir).is_err());
    }
    #[test]
    fn compatibility_channel_ten_is_not_silently_general_midi() {
        let bytes = seq(&[0, 0x99, 60, 100, 96, 0x89, 60, 0, 0, 0xff, 0x2f, 0]);
        let settings = Settings { source_selection: Some(select_source(&bytes, SourceProfile::SonySeqV1, Some(0), None).unwrap()), ignore_unsupported: true, ..Default::default() };
        assert!(settings.validate_playback(&decode_source(&bytes, &settings).unwrap()).unwrap_err().contains("channel 10"));
    }
    #[test]
    fn converted_profile_alias_migrates_without_changing_selection_or_psx_payload() {
        let mut bytes = converted_seq(60);
        bytes.extend(converted_seq(72));
        let mut settings = Settings {
            source_selection: Some(select_source(&bytes, SourceProfile::ConvertedSeqLe32V1, None, Some(1)).unwrap()),
            ..Default::default()
        };
        settings.source_selection.as_mut().unwrap().extra.insert("future".into(), serde_json::json!({"x": 42}));
        let canonical = serde_json::to_value(&settings).unwrap();
        let mut legacy = canonical.clone();
        let legacy_id = "legacy-seqcomb-sepcomb-v1";
        legacy["source_selection"]["profile"] = legacy_id.into();
        let restored: Settings = serde_json::from_value(legacy).unwrap();
        assert_eq!(restored, settings);
        assert_eq!(serde_json::to_value(&restored).unwrap(), canonical);
        assert_eq!(SourceProfile::from_id(legacy_id).unwrap().id(), "converted-seq-le32-v1");
        for profile in [SourceProfile::SonySeqV1, SourceProfile::SonySepV0, SourceProfile::ConvertedSeqLe32V1] {
            assert_eq!(SourceProfile::from_id(profile.id()).unwrap(), profile);
        }
        assert!(SourceProfile::from_id("converted-seq-le32-v2").is_err());
        let ir = decode_source(&bytes, &restored).unwrap();
        assert_eq!(ir.source_profile.as_deref(), Some("converted-seq-le32-v1"));
        assert_eq!(ir.source_events[0].data[0], 72);
        let bank = Uuid::from_u128(1);
        assert_eq!(crate::psx_sequence::sequence_payload(&ir, &restored, bank).unwrap().1,
            crate::psx_sequence::sequence_payload(&decode_source(&bytes, &settings).unwrap(), &settings, bank).unwrap().1);
        let mut reordered = converted_seq(72);
        reordered.extend(converted_seq(60));
        assert!(decode_source(&reordered, &restored).unwrap_err().contains("reselect"));
    }
    #[test]
    fn legacy_settings_json_and_future_selection_fields_are_preserved() {
        let value = serde_json::to_value(Settings::default()).unwrap(); assert!(value.get("source_selection").is_none());
        let before: Settings = serde_json::from_value(value.clone()).unwrap(); assert_eq!(serde_json::to_value(before).unwrap(), value);
        let bytes = seq(&score(60));
        let mut settings = Settings { source_selection: Some(select_source(&bytes, SourceProfile::SonySeqV1, Some(0), None).unwrap()), ..Default::default() };
        settings.source_selection.as_mut().unwrap().extra.insert("future".into(), serde_json::json!({"x":42}));
        let saved = serde_json::to_vec(&settings).unwrap(); let restored: Settings = serde_json::from_slice(&saved).unwrap(); assert_eq!(restored, settings);
        let mut future = restored; future.source_selection.as_mut().unwrap().schema_version = 99; assert!(future.validate().is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets, sound_bank};

    #[test]
    fn midi_interpretation_migration_preserves_old_settings_until_explicit_upgrade() {
        let mut old = serde_json::to_value(Settings::default()).unwrap();
        old.as_object_mut().unwrap().remove("midi_profile");
        old["ignore_unsupported"] = true.into();
        old["future_settings"] = serde_json::json!({"keep": 27});
        let legacy: Settings = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(legacy.midi_profile, crate::midi::MidiProfile::LegacyV1);
        assert_eq!(serde_json::to_value(&legacy).unwrap(), old);
        assert_eq!(Settings::default().midi_profile, crate::midi::MidiProfile::MusicalV2);

        let root = crate::workspace::tests::temp("midi-interpretation-migration");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let events = [0, 0xb0, 101, 0, 0, 100, 0, 0, 6, 12, 0, 0x90, 60, 100,
            96, 0x80, 60, 0, 0, 0xff, 0x2f, 0];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\0\x60MTrk".to_vec();
        bytes.extend((events.len() as u32).to_be_bytes());
        bytes.extend(events);
        std::fs::write(root.join("assets/rpn.mid"), &bytes).unwrap();
        let legacy_ir = decode_source(&bytes, &legacy).unwrap();
        assert_eq!(legacy_ir.diagnostics.iter().filter(|d| d.unsupported).count(), 3);
        let id = assets::commit(prepare(&root, "assets/rpn.mid", "assets/rpn.epokasset",
            legacy.clone(), None, false).unwrap()).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap();
        let old_key = assets::cache_key(&record.meta);
        let mut upgraded = legacy;
        upgraded.midi_profile = crate::midi::MidiProfile::MusicalV2;
        upgraded.ignore_unsupported = false;
        let ir = decode_source(&bytes, &upgraded).unwrap();
        upgraded.validate_playback(&ir).unwrap();
        assert!(ir.events.iter().any(|e| matches!(e.kind,
            crate::sequence_ir::EventKind::Parameter { channel: 0, parameter: 0, value: 1200 })));
        let actual = assets::commit(prepare(&root, "assets/rpn.mid", "assets/rpn.epokasset",
            upgraded, Some(record), true).unwrap()).unwrap();
        assert_eq!(actual, id);
        let package = assets::Package::load(&root.join("assets/rpn.epokasset")).unwrap();
        assert_eq!(package.source, bytes);
        assert_ne!(assets::cache_key(&package.meta), old_key);
        assert_eq!(package.meta.settings.sequence().unwrap().extra["future_settings"], serde_json::json!({"keep":27}));

        // Musical v2 never turns unsupported controls into successful no-ops.
        let mut unknown = bytes;
        unknown[24] = 119;
        let settings = Settings { ignore_unsupported: true, ..Default::default() };
        let error = settings.validate_playback(&decode_source(&unknown, &settings).unwrap()).unwrap_err();
        assert!(error.contains("CC 119=0") && error.contains("track 1, tick 0") && error.contains("Legacy v1 exception"));
        let legacy = Settings { midi_profile: crate::midi::MidiProfile::LegacyV1, ..settings };
        legacy.validate_playback(&decode_source(&unknown, &legacy).unwrap()).unwrap();
    }

    #[test]
    fn midi_assets_preserve_source_identity_unknowns_moves_and_bank_dependencies() {
        let root = crate::workspace::tests::temp("sequence-assets");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let sample = assets::commit(
            assets::prepare(
                &root,
                "assets/tone.wav",
                "assets/tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let bank_settings = sound_bank::Settings {
            programs: vec![sound_bank::Program {
                program: 0,
                drum_key: None,
                zones: vec![sound_bank::Zone::new(sample)],
                extra: Default::default(),
            }],
            provenance: "Original Epok test signal, MIT".into(),
            ..Default::default()
        };
        let bank =
            sound_bank::create(&root, "assets/bank.epokasset", bank_settings.clone()).unwrap();
        let bytes = crate::midi::fixture();
        std::fs::write(root.join("assets/song.mid"), &bytes).unwrap();
        let mut settings = Settings {
            sound_bank: Some(bank),
            ..Default::default()
        };
        settings
            .extra
            .insert("future".into(), serde_json::json!({"value":42}));
        settings.envelope_extra.insert("outer".into(), 17.into());
        let id = assets::commit(
            prepare(
                &root,
                "assets/song.mid",
                "assets/song.epokasset",
                settings.clone(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap();
        assert_eq!(record.meta.kind, assets::Kind::MusicSequence);
        assert_eq!(record.meta.settings.sequence().unwrap(), &settings);
        let selected = resolve_bank(&root, &settings, &index).unwrap();
        selected
            .meta
            .settings
            .sound_bank()
            .unwrap()
            .validate_sequence(&crate::midi::parse(&bytes).unwrap(), &index)
            .unwrap();
        assert!(
            resolve_bank(&root, &Settings::default(), &index)
                .unwrap_err()
                .contains("SoundBank")
        );
        assert!(
            assets::dependencies(&root, bank)
                .unwrap()
                .contains(&"assets/song.epokasset".into())
        );
        assert!(
            assets::dependencies(&root, sample)
                .unwrap()
                .contains(&"assets/bank.epokasset".into())
        );
        assert!(assets::trash(&root, index.resolve(sample).unwrap()).is_err());
        assets::move_asset(&root, record, "assets/moved.epokasset").unwrap();
        std::fs::remove_file(root.join("assets/song.mid")).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap();
        settings.voice_limit = Some(12);
        assert_eq!(
            assets::commit(
                prepare(
                    &root,
                    "",
                    "assets/moved.epokasset",
                    settings,
                    Some(record),
                    true
                )
                .unwrap()
            )
            .unwrap(),
            id
        );
        let package = assets::Package::load(&record.path).unwrap();
        assert_eq!(package.source, bytes);
        assert_eq!(
            package.meta.settings.sequence().unwrap().extra["future"]["value"],
            42
        );
        let duplicate = assets::duplicate(record, &root.join("assets/copy.epokasset")).unwrap();
        assert_ne!(duplicate, id);
        assert_eq!(
            assets::Package::load(&root.join("assets/copy.epokasset"))
                .unwrap()
                .meta
                .settings
                .sequence()
                .unwrap()
                .sound_bank,
            Some(bank)
        );
        let mut changed = bank_settings;
        changed.programs[0].zones[0].gain = 0.5;
        let old_key = assets::cache_key(&selected.meta);
        let candidate = assets::prepare_portable(
            &root,
            "",
            "assets/bank.epokasset",
            crate::import_settings::Settings::SoundBank(changed),
            Some(selected),
            true,
        )
        .unwrap();
        assert_ne!(assets::cache_key(&candidate.package.meta), old_key);
        assets::commit(candidate).unwrap();
    }
    #[test]
    fn midi_bank_ranges_drums_and_unsupported_policy_are_explicit() {
        let ir = crate::midi::parse(&crate::midi::fixture()).unwrap();
        let empty = sound_bank::Settings::default();
        assert!(
            empty
                .validate_sequence(&ir, &assets::Index::default())
                .unwrap_err()
                .contains("program 0")
        );
        let mut settings = Settings {
            loop_mode: LoopMode::Markers,
            ..Default::default()
        };
        assert!(
            settings
                .validate_playback(&ir)
                .unwrap_err()
                .contains("loop_start")
        );
        settings.loop_mode = LoopMode::Off;
        settings.load_mode = LoadMode::Stream;
        assert!(
            settings
                .validate_playback(&ir)
                .unwrap_err()
                .contains("Resident")
        );
        let mut ir = ir;
        ir.diagnostics.push(crate::sequence_ir::Diagnostic {
            track: 0,
            tick: 0,
            message: "Unsupported SysEx".into(),
            unsupported: true,
        });
        settings.load_mode = LoadMode::Resident;
        assert!(
            settings
                .validate_playback(&ir)
                .unwrap_err()
                .contains("explicitly")
        );
        settings.ignore_unsupported = true;
        settings.validate_playback(&ir).unwrap();
        let mut zone = sound_bank::Zone::new(Uuid::new_v4());
        zone.key_range = [48, 72];
        zone.velocity_range = [50, 100];
        assert!(zone.matches(60, 60));
        assert!(!zone.matches(60, 1));
        assert!(!zone.matches(80, 60));
        let mut bank = sound_bank::Settings {
            programs: vec![sound_bank::Program {
                program: 0,
                drum_key: Some(60),
                zones: vec![zone],
                extra: Default::default(),
            }],
            ..Default::default()
        };
        assert!(bank.zone(9, 0, 60, 60).is_some());
        assert!(bank.zone(0, 0, 60, 60).is_none());
        bank.programs[0].zones[0].envelope.attack_ms = f32::NAN;
        assert!(bank.validate().is_err());
    }
}
