//! Explicit PSX cook from original audio snapshots and the neutral musical IR.
//! Files use little endian integers, relative offsets, and independently versioned headers.
use crate::{assets, audio_import, sequence_stream};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use uuid::Uuid;

pub const PROFILE: &str = "psx-sequence-v1";
pub const MUSICAL_PROFILE: &str = "psx-sequence-v2";
pub const SEQUENCE_HEADER: usize = 40;
pub const BANK_HEADER: usize = 32;
pub const SAMPLE_RECORD: usize = 24;
pub const ZONE_RECORD: usize = 32;
pub const SEQUENCE_BUDGET: usize = 256 * 1024;

pub fn cook_key(root: &Path, meta: &assets::Metadata) -> Result<String, String> {
    let index = assets::scan(root, &mut Default::default());
    let mut inputs = BTreeMap::from([(format!("asset:{}", meta.id), assets::cache_key(meta))]);
    let bank = match meta.kind {
        assets::Kind::MusicSequence => {
            let settings = meta.settings.sequence()?;
            let record = crate::sequence::resolve_bank(root, settings, &index)?;
            if record.meta.settings.sound_bank()?.library.is_some() {
                return Ok(identity(&crate::psx_library_asset::inputs(meta,&record.meta)?));
            }
            if settings.sound_bank.is_none() {
                inputs.insert("default-sound-bank".into(), record.meta.id.to_string());
            }
            inputs.insert(
                format!("asset:{}", record.meta.id),
                assets::cache_key(&record.meta),
            );
            record.meta.settings.sound_bank()?
        }
        assets::Kind::SoundBank => meta.settings.sound_bank()?,
        _ => return Err("PSX sequence cook key requires MusicSequence or SoundBank".into()),
    };
    for id in bank.dependencies() {
        inputs.insert(
            format!("asset:{id}"),
            assets::cache_key(&index.resolve(id)?.meta),
        );
    }
    Ok(identity(&inputs))
}

pub fn identity(inputs: &BTreeMap<String, String>) -> String {
    static IMPLEMENTATION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let implementation = IMPLEMENTATION.get_or_init(|| {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        for source in [
            include_bytes!("psx_sequence.rs").as_slice(),
            include_bytes!("sequence_stream.rs").as_slice(),
            include_bytes!("sequence_ir.rs").as_slice(),
            include_bytes!("midi.rs").as_slice(),
            include_bytes!("midi_controls.rs").as_slice(),
            include_bytes!("../runtime/instrument_preparation.hpp").as_slice(),
            include_bytes!("sound_bank.rs").as_slice(),
            include_bytes!("sequence.rs").as_slice(),
            include_bytes!("sequence_compat.rs").as_slice(),
            include_bytes!("bank_compat.rs").as_slice(),
            include_bytes!("vab_import.rs").as_slice(),
            include_bytes!("audio_import.rs").as_slice(),
            include_bytes!("audio_ir.rs").as_slice(),
            include_bytes!("audio_decode.rs").as_slice(),
            include_bytes!("sf2.rs").as_slice(),
            include_bytes!("instrument_ir.rs").as_slice(),
            include_bytes!("instrument_selection.rs").as_slice(),
            include_bytes!("instrument_samples.rs").as_slice(),
            include_bytes!("instrument_voice.rs").as_slice(),
            include_bytes!("instrument_modulation.rs").as_slice(),
            include_bytes!("instrument_dsp.rs").as_slice(),
            include_bytes!("spu_encoder.rs").as_slice(),
            include_bytes!("psx_library.rs").as_slice(),
            include_bytes!("psx_music_settings.rs").as_slice(),
            include_bytes!("psx_music_optimizer.rs").as_slice(),
            include_bytes!("psx_library_wire.rs").as_slice(),
            include_bytes!("psx_library_asset.rs").as_slice(),
            include_bytes!("psx_loop_quality.rs").as_slice(),
            include_bytes!("../Cargo.lock").as_slice(),
            include_bytes!("../runtime/sequence_kernel.hpp").as_slice(),
            include_bytes!("../runtime/sequence_data.hpp").as_slice(),
            include_bytes!("../runtime/instrument_bank.hpp").as_slice(),
            include_bytes!("../runtime/instrument_allocator.hpp").as_slice(),
            include_bytes!("../runtime/instrument_synth.hpp").as_slice(),
            include_bytes!("../runtime/instrument_reverb.hpp").as_slice(),
            include_bytes!("../runtime/sequence_instrument_service.hpp").as_slice(),
            include_bytes!("../runtime/sequence_service.hpp").as_slice(),
            include_bytes!("../runtime/sequence_tables.hpp").as_slice(),
            include_bytes!("../runtime/sequence_lock.hpp").as_slice(),
            include_bytes!("../runtime/sequence_clock.hpp").as_slice(),
        ] {
            digest.update(source);
        }
        format!("{:x}", digest.finalize())
    });
    assets::hash(
        format!(
            "{PROFILE}:builtin:{implementation}:{}",
            serde_json::to_string(inputs).unwrap()
        )
        .as_bytes(),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub bytes: Vec<u8>,
    pub rate: u32,
    pub frames: u32,
    /// Decoded frame coordinates include the initial silent SPU block.
    pub loop_region: Option<[u32; 2]>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    pub sample: u16,
    pub program: u8,
    pub drum_key: u8,
    pub key_range: [u8; 2],
    pub velocity_range: [u8; 2],
    pub root_key: u8,
    pub cents: i16,
    pub gain: u16,
    pub pan: i16,
    pub attack_ms: u32,
    pub decay_ms: u32,
    pub sustain: u16,
    pub release_ms: u32,
}
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub target: &'static str,
    pub profile: &'static str,
    pub resolved_mode: &'static str,
    pub bank_mode: &'static str,
    pub payload_version: u16,
    pub endianness: &'static str,
    pub sequence_bytes: usize,
    pub bank_bytes: usize,
    pub sample_main_ram_bytes: usize,
    pub prepared_start_states: u16,
    pub prepared_start_references: u32,
    pub elided_noop_events: usize,
    pub spu_ram_bytes: usize,
    pub package_bytes: u64,
    pub voice_limit: u16,
    pub peak_polyphony: u32,
    pub warnings: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Bank {
    pub id: Uuid,
    pub samples: Vec<Sample>,
    pub zones: Vec<Zone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library: Option<LibraryMetadata>,
    #[serde(skip)]
    pub payload: Vec<u8>,
    pub inputs: BTreeMap<String, String>,
    pub package_bytes: u64,
    pub warnings: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub zones: Vec<crate::psx_library::Zone>,
    pub report: crate::psx_library::Report,
}
pub struct Cooked {
    pub payload: Vec<u8>,
    pub bank: Bank,
    pub report: Report,
    pub inputs: BTreeMap<String, String>,
}

#[derive(Default)]
pub struct Staged {
    pub declarations: String,
    pub descriptors: BTreeMap<Uuid, String>,
    pub inputs: BTreeMap<String, String>,
    pub outputs: Vec<crate::playback_staging::ResourceOutput>,
    pub spu_bytes: usize,
}
pub fn stage(
    root: &Path,
    scene: &crate::scene::Scene,
    build: &Path,
    index: &assets::Index,
) -> Result<Staged, String> {
    let mut staged = Staged::default();
    let mut banks = BTreeMap::new();
    let mut reports = BTreeMap::new();
    let mut reverb = None;
    let declaration = |name: &str, bytes: &[u8]| {
        let mut text = format!("alignas(4) inline const uint8_t {name}[] = {{\n");
        for chunk in bytes.chunks(32) {
            text.push_str(
                &chunk
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
            text.push_str(",\n");
        }
        text.push_str("};\n");
        text
    };
    for (i, id) in crate::audio::clip_ids(scene).into_iter().enumerate() {
        let record = index.resolve(id)?;
        if record.meta.kind != assets::Kind::MusicSequence {
            continue;
        }
        let package = assets::Package::load(&record.path)?;
        if package.meta.id != id
            || assets::cache_key(&package.meta) != assets::cache_key(&record.meta)
        {
            return Err(format!(
                "MusicSequence {id} changed during staging; retry build"
            ));
        }
        let cooked = cook(root, &package, index)?;
        // Cooking a large library gives reimport time to commit a new revision.
        // Verify every selected asset again before emitting any scene output.
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        crate::psx_library_asset::verify_record(record, &cancelled)?;
        for (input, expected) in &cooked.inputs {
            if let Some(asset) = input.strip_prefix("asset:") {
                let id = Uuid::parse_str(asset).map_err(|e| e.to_string())?;
                let dependency = index.resolve(id)?;
                if assets::cache_key(&dependency.meta) != *expected { return Err("Audio dependency changed during staging; retry build".into()); }
                crate::psx_library_asset::verify_record(dependency, &cancelled)?;
            }
        }
        let key = identity(&cooked.inputs);
        staged.inputs.extend(cooked.inputs.clone());
        staged.inputs.insert(format!("audio-cook:{id}"), key);
        let next_bank = banks.len();
        let bank_index = *banks.entry(cooked.bank.id).or_insert(next_bank);
        if bank_index == next_bank {
            staged.spu_bytes += cooked.report.spu_ram_bytes;
            if let Some(library) = &cooked.bank.library
                && library.report.recipe.effects == crate::psx_music_settings::Effects::Room
            {
                let depth = library.report.recipe.reverb_depth_q15();
                if reverb.is_some_and(|previous| previous != depth) {
                    return Err("Resident PSX banks request conflicting global reverb depths; use one Room configuration for this scene".into());
                }
                if reverb.replace(depth).is_none() { staged.spu_bytes += crate::psx_music_settings::ROOM_REVERB_BYTES as usize; }
            }
            if staged.spu_bytes > audio_import::SPU_BUDGET {
                return Err(format!(
                    "Resident PSX SoundBanks require {} SPU bytes; aggregate budget {}. Banks remain Resident; reduce their samples.",
                    staged.spu_bytes,
                    audio_import::SPU_BUDGET
                ));
            }
            let name = format!("sequence_bank_data_{bank_index}");
            staged
                .declarations
                .push_str(&declaration(&name, &cooked.bank.payload));
            staged.declarations.push_str(&format!(
                "inline psx_audio::Bank sequence_bank_{bank_index}{{{name},{}}};\n",
                cooked.bank.payload.len()
            ));
            let path = format!("audio/{}.epsb", cooked.bank.id);
            crate::project::write_changed(&build.join(&path), &cooked.bank.payload)?;
            staged
                .outputs
                .push(crate::playback_staging::ResourceOutput {
                    path,
                    signature: assets::hash(&cooked.bank.payload),
                    inputs: cooked.bank.inputs.clone(),
                });
        }
        let name = format!("sequence_data_{i}");
        staged
            .declarations
            .push_str(&declaration(&name, &cooked.payload));
        let prepared = if cooked.report.prepared_start_states>0 {
            staged.declarations.push_str(&format!("inline instrument::preparation::Storage<{},{},{}> sequence_prepared_{i};\n",cooked.report.prepared_start_states,(cooked.payload.len()-SEQUENCE_HEADER)/12,cooked.report.prepared_start_references));
            format!("&sequence_prepared_{i}.cache")
        } else { "nullptr".into() };
        staged.declarations.push_str(&format!("inline const psx_audio::Sequence sequence_asset_{i}{{{name},{},&sequence_bank_{bank_index},{prepared}}};\n", cooked.payload.len()));
        staged.descriptors.insert(
            id,
            format!("{{nullptr,0,0,false,nullptr,&sequence_asset_{i}}}"),
        );
        let path = format!("audio/{id}.epsq");
        crate::project::write_changed(&build.join(&path), &cooked.payload)?;
        staged
            .outputs
            .push(crate::playback_staging::ResourceOutput {
                path,
                signature: assets::hash(&cooked.payload),
                inputs: staged.inputs.clone(),
            });
        reports.insert(id, cooked.report);
    }
    if !reports.is_empty() {
        let report = serde_json::to_vec_pretty(&reports).map_err(|e| e.to_string())?;
        let path = "audio/sequence-report.json".to_string();
        crate::project::write_changed(&build.join(&path), &report)?;
        staged
            .outputs
            .push(crate::playback_staging::ResourceOutput {
                path,
                signature: assets::hash(&report),
                inputs: staged.inputs.clone(),
            });
    }
    Ok(staged)
}

fn u16le(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn u32le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Remove only assignments that have no effect under the versioned kernel's
/// initial channel state. The authoritative IR/ledger and every note remain
/// intact. LoopStart snapshots the same state before every repeated traversal.
fn compact_noop_controls(events: &mut Vec<sequence_stream::Event>) {
    #[derive(Clone,Copy)]
    struct Channel { program:u8, bank:u16, bend:u16, parameters:[u16;3], cc:[Option<u8>;128] }
    let mut initial=Channel { program:0,bank:0,bend:8192,parameters:[200,8192,64],cc:[None;128] };
    for (cc,value) in [(1,0),(7,100),(10,64),(11,127),(64,0),(66,0)] { initial.cc[cc]=Some(value); }
    // CC91's first zero must remain: it suppresses a bank's authored send.
    let mut channels=[initial;16];
    fn assign<T:Copy+Eq>(old: &mut T,value:T)->bool { let changed=*old!=value;*old=value;changed }
    events.retain(|event| {
        let Some(channel)=channels.get_mut(event.channel as usize) else { return true; };
        match event.op {
            2|10 => {
                let bank=if event.op==2 {0} else {event.value as u16};
                let changed=channel.program!=event.a || channel.bank!=bank;
                channel.program=event.a;channel.bank=bank;changed
            },
            4=>assign(&mut channel.bend,event.value as u16),
            9=>channel.parameters.get_mut(event.a as usize).is_none_or(|old|assign(old,event.value as u16)),
            3=>match event.a {
                1|7|10|11|91=>assign(&mut channel.cc[event.a as usize],Some(event.b)),
                64|66=>assign(&mut channel.cc[event.a as usize],Some(if event.b>=64 {127} else {0})),
                // These zero depths are defined no-ops in this target profile.
                92|93|95=>event.b!=0,
                121=>{
                    channel.bend=8192;
                    for(cc,value)in[(1,0),(11,127),(64,0),(66,0)]{channel.cc[cc]=Some(value);}
                    true // Keep the reset's note/pedal behavior.
                },
                _=>true,
            },
            _=>true,
        }
    });
}

/// With no preceding notes, the zero-tick channel setup can run once before
/// Whole-loop capture. Restoring that snapshot is equivalent to restoring the
/// defaults and replaying the same assignments on every loop, without that IRQ
/// burst. Do not move a note, a later-tick event, or a marker-loop boundary.
fn hoist_whole_loop_setup(events: &mut [sequence_stream::Event]) -> usize {
    if events.first().is_none_or(|e| e.op!=6 || e.tick!=0) { return 0; }
    let setup=events.iter().skip(1).take_while(|e| e.tick==0 && matches!(e.op,2|3|4|5|9|10)).count();
    events[..=setup].rotate_left(1);
    setup
}

fn library_events(ir: &crate::sequence_ir::SequenceIr,settings: &crate::sequence::Settings)
    -> Result<Vec<sequence_stream::Event>,String> {
    let mut events=sequence_stream::events(ir,settings)?;
    compact_noop_controls(&mut events);
    if settings.loop_mode==crate::sequence::LoopMode::Whole { hoist_whole_loop_setup(&mut events); }
    Ok(events)
}

pub fn sequence_payload(
    ir: &crate::sequence_ir::SequenceIr,
    settings: &crate::sequence::Settings,
    bank: Uuid,
) -> Result<(Vec<sequence_stream::Event>, Vec<u8>), String> {
    sequence_payload_impl(ir,settings,bank,false)
}
pub(crate) fn library_payload(ir: &crate::sequence_ir::SequenceIr,settings: &crate::sequence::Settings,
    bank: Uuid) -> Result<(Vec<sequence_stream::Event>,Vec<u8>),String> {
    sequence_payload_impl(ir,settings,bank,true)
}
fn sequence_payload_impl(ir: &crate::sequence_ir::SequenceIr,settings: &crate::sequence::Settings,
    bank: Uuid,compact: bool) -> Result<(Vec<sequence_stream::Event>,Vec<u8>),String> {
    if settings.voices() > 24 {
        return Err(format!(
            "PSX has 24 physical sample voices; requested music ceiling {}. Edit Voice Limit explicitly.",
            settings.voices()
        ));
    }
    let events = if compact { library_events(ir,settings)? } else { sequence_stream::events(ir,settings)? };
    let size = SEQUENCE_HEADER + events.len() * 12;
    if size > SEQUENCE_BUDGET {
        return Err(format!(
            "PSX sequence requires {size} resident bytes; profile limit is {SEQUENCE_BUDGET}. Reduce event density or split the sequence."
        ));
    }
    let mut out = b"EPSQ".to_vec();
    u16le(&mut out, sequence_stream::payload_version(&events));
    u16le(&mut out, SEQUENCE_HEADER as u16);
    u16le(&mut out, ir.ppqn);
    u16le(&mut out, settings.voices());
    u32le(&mut out, events.len() as u32);
    u32le(&mut out, ir.duration_ticks);
    u32le(&mut out, 0);
    out.extend_from_slice(bank.as_bytes());
    for event in &events {
        u32le(&mut out, event.tick);
        out.extend_from_slice(&[event.op, event.channel, event.a, event.b]);
        u32le(&mut out, event.value);
    }
    Ok((events, out))
}

fn sample(
    source: &[u8],
    settings: &audio_import::Settings,
    region: Option<[u32; 2]>,
) -> Result<(Sample, Vec<String>), String> {
    // Sample residency is owned by the bank, independently of this AudioClip's own use.
    let mut settings = settings.clone();
    settings.load_mode = audio_import::LoadMode::Resident;
    settings.validate_psx()?;
    let original = crate::audio_ir::DecodedAudioIr::decode(source)?;
    let (start, end) = audio_import::trim_range(&original.info, &settings)?;
    let rate = settings.rate();
    let frames =
        ((end - start) as u64 * rate as u64).div_ceil(original.info.sample_rate as u64) as usize;
    if (frames.div_ceil(28) + 2) * 16 > audio_import::SPU_BUDGET {
        return Err(
            "PSX SoundBank sample exceeds SPU RAM; shorten source trim or lower its PSX rate"
                .into(),
        );
    }
    let mono = original.mono();
    let pcm = audio_import::resample_pcm(&mono[start..end], original.info.sample_rate, rate);
    let peak = pcm.iter().fold(0f32, |a, b| a.max(b.abs()));
    let gain = if settings.normalize && peak > 0. {
        0.95 / peak
    } else {
        1.
    };
    let mut pcm = pcm
        .iter()
        .map(|v| (v * gain * 32767.).round().clamp(-32768., 32767.) as i16)
        .collect::<Vec<_>>();
    let mut warnings = Vec::new();
    let resolved = if let Some([a, b]) = region {
        if (a as usize) < start || b as usize > end {
            return Err("PSX SoundBank sample loop lies outside its saved source trim".into());
        }
        let to_target =
            |v: u32| (v as f64 - start as f64) * rate as f64 / original.info.sample_rate as f64;
        let target_a = to_target(a);
        let target_b = to_target(b);
        // Round outward to block boundaries. The terminal partial block is zero padded,
        // exactly as for sampled clips. Original markers remain in authoring unchanged.
        let a = (target_a / 28.).floor() as usize * 28;
        let b = (target_b / 28.).ceil() as usize * 28;
        if a >= b || b > frames.div_ceil(28) * 28 {
            return Err("PSX SoundBank loop cannot be represented within the cooked sample".into());
        }
        if a as f64 != target_a || b as f64 != target_b {
            warnings.push(format!("Sample loop source {:?}: PSX target frames {target_a:.3}..{target_b:.3} resolved outward to {a}..{b} (28-frame ADPCM blocks)", region.unwrap()));
        }
        pcm.resize(b, 0);
        Some([a as u32 + 28, b as u32 + 28])
    } else {
        None
    };
    let mut bytes = audio_import::encode_blocks(&pcm, resolved.map(|r| (r[0] as usize - 28) / 28));
    let decoded_frames = (bytes.len() / 16 - 1) as u32 * 28;
    bytes.resize(bytes.len().div_ceil(64) * 64, 0);
    Ok((
        Sample {
            bytes,
            rate,
            frames: decoded_frames,
            loop_region: resolved,
        },
        warnings,
    ))
}

pub fn sample_key(sample: &Sample) -> String {
    assets::hash(
        format!(
            "{}:{}:{:?}:{}",
            sample.rate,
            sample.frames,
            sample.loop_region,
            assets::hash(&sample.bytes)
        )
        .as_bytes(),
    )
}

pub fn bank(root: &Path, record: &assets::Record, index: &assets::Index) -> Result<Bank, String> {
    let package = assets::Package::load(&record.path)?;
    if package.meta.id != record.meta.id
        || assets::cache_key(&package.meta) != assets::cache_key(&record.meta)
    {
        return Err("SoundBank changed during resource selection; retry cook".into());
    }
    let source_bank = crate::bank_compat::decode(&package)?;
    let settings = source_bank.resolve()?;
    settings.validate()?;
    if settings.load_mode == audio_import::LoadMode::Stream {
        return Err("PSX SoundBank Stream is unavailable in this profile. Select Resident/Auto explicitly or use sampled streamed music.".into());
    }
    if settings.programs.is_empty() {
        return Err("PSX SoundBank needs at least one program/zone".into());
    }
    let mut bank = Bank {
        id: package.meta.id,
        samples: vec![],
        zones: vec![],
        library: None,
        payload: vec![],
        inputs: BTreeMap::from([(
            format!("asset:{}", package.meta.id),
            assets::cache_key(&package.meta),
        )]),
        package_bytes: assets::read_bounded(&record.path)?.len() as u64,
        warnings: vec![],
    };
    let mut sources = BTreeMap::new();
    for id in settings.dependencies() {
        let record = index.resolve(id)?;
        let sample = assets::Package::load(&record.path)?;
        if sample.meta.id != id
            || assets::cache_key(&sample.meta) != assets::cache_key(&record.meta)
        {
            return Err(format!(
                "SoundBank sample {id} changed during resource selection; retry cook"
            ));
        }
        sample.meta.settings.audio()?;
        bank.inputs
            .insert(format!("asset:{id}"), assets::cache_key(&sample.meta));
        bank.package_bytes += assets::read_bounded(&record.path)?.len() as u64;
        sources.insert(id, sample);
    }
    let key = identity(&bank.inputs);
    let cache = root.join(".epok/imported").join(&key);
    if std::fs::metadata(cache.join("bank.epokcache")).is_ok_and(|m| m.len() <= 4 * 1024 * 1024)
        && let Ok(bytes) = assets::read_bounded(&cache.join("bank.epokcache"))
        && let Ok(checksum) = std::fs::read_to_string(cache.join("checksum"))
        && checksum == assets::hash(&bytes)
        && let Ok(mut cached) = serde_json::from_slice::<Bank>(&bytes)
        && cached.id == bank.id
        && cached.inputs == bank.inputs
        && cached.library.is_none()
        && cached.samples.len() <= crate::sound_bank::MAX_ZONES
        && cached.zones.len() <= crate::sound_bank::MAX_ZONES
        && cached
            .samples
            .iter()
            .all(|s| s.bytes.len() >= 64 && s.bytes.len().is_multiple_of(64))
        && cached.samples.iter().map(|s| s.bytes.len()).sum::<usize>() <= audio_import::SPU_BUDGET
        && cached
            .zones
            .iter()
            .all(|z| (z.sample as usize) < cached.samples.len())
    {
        cached.payload = bank_payload(&cached);
        assets::replace_cache(&cache.join("bank.epsb"), &cached.payload)?;
        bank_summary(root, &package.meta, &cached)?;
        return Ok(cached);
    }
    let mut dedup = BTreeMap::new();
    for program in &settings.programs {
        for z in &program.zones {
            let package = &sources[&z.sample];
            let (cooked, warnings) = sample(
                &package.source,
                package.meta.settings.audio()?,
                z.sample_loop,
            )?;
            for warning in warnings {
                bank.warnings
                    .push(format!("Sample {}: {warning}", z.sample));
            }
            let key = sample_key(&cooked);
            let sample = *dedup.entry(key).or_insert_with(|| {
                let index = bank.samples.len() as u16;
                bank.samples.push(cooked);
                index
            });
            bank.zones.push(Zone {
                sample,
                program: program.program,
                drum_key: program.drum_key.unwrap_or(255),
                key_range: z.key_range,
                velocity_range: z.velocity_range,
                root_key: z.root_key,
                cents: (z.fine_tune_cents * 100.).round() as i16,
                gain: (z.gain * 4096.).round() as u16,
                pan: (z.pan * 16384.).round() as i16,
                attack_ms: z.envelope.attack_ms.ceil() as u32,
                decay_ms: z.envelope.decay_ms.ceil() as u32,
                sustain: (z.envelope.sustain * 32767.).round() as u16,
                release_ms: z.envelope.release_ms.ceil() as u32,
            });
        }
    }
    let spu: usize = bank.samples.iter().map(|s| s.bytes.len()).sum();
    if spu > audio_import::SPU_BUDGET {
        return Err(format!(
            "PSX SoundBank uses {spu} SPU bytes after deduplication; budget {}. Bank Load Mode was not changed.",
            audio_import::SPU_BUDGET
        ));
    }
    bank.warnings.push("PSX profile: mono samples; linear software ADSR at 1 ms service resolution; time values rounded upward, sustain Q15, tuning 0.01 cent, gain Q12 and pan Q14. Hardware interpolation differs from Source Preview.".into());
    bank.payload = bank_payload(&bank);
    let bytes = serde_json::to_vec(&bank).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    assets::replace_cache(&cache.join("bank.epokcache"), &bytes)?;
    assets::replace_cache(&cache.join("checksum"), assets::hash(&bytes).as_bytes())?;
    assets::replace_cache(&cache.join("bank.epsb"), &bank.payload)?;
    bank_summary(root, &package.meta, &bank)?;
    Ok(bank)
}

fn bank_summary(root: &Path, meta: &assets::Metadata, bank: &Bank) -> Result<(), String> {
    let spu: usize = bank.samples.iter().map(|sample| sample.bytes.len()).sum();
    let summary = root.join(".epok/imported").join(assets::cache_key(meta));
    std::fs::create_dir_all(&summary).map_err(|e| e.to_string())?;
    let summary_bytes = serde_json::to_vec(&serde_json::json!({"inputs":bank.inputs,"profile":PROFILE,"bank_bytes":bank.payload.len(),"spu_ram_bytes":spu,"warnings":bank.warnings})).map_err(|e| e.to_string())?;
    assets::replace_cache(&summary.join("Bank.epokcache"), &summary_bytes)
}

fn bank_payload(bank: &Bank) -> Vec<u8> {
    let zones_offset = BANK_HEADER + bank.samples.len() * SAMPLE_RECORD;
    let data_offset = (zones_offset + bank.zones.len() * ZONE_RECORD).div_ceil(64) * 64;
    let total = data_offset + bank.samples.iter().map(|s| s.bytes.len()).sum::<usize>();
    let mut out = b"EPSB".to_vec();
    u16le(&mut out, 1);
    u16le(&mut out, BANK_HEADER as u16);
    u16le(&mut out, bank.samples.len() as u16);
    u16le(&mut out, bank.zones.len() as u16);
    for value in [BANK_HEADER, zones_offset, data_offset, total, 0] {
        u32le(&mut out, value as u32);
    }
    let mut offset = data_offset;
    for s in &bank.samples {
        for value in [
            offset as u32,
            s.bytes.len() as u32,
            s.rate,
            s.frames,
            s.loop_region.map_or(0, |r| r[0]),
            s.loop_region.map_or(0, |r| r[1]),
        ] {
            u32le(&mut out, value);
        }
        offset += s.bytes.len();
    }
    for z in &bank.zones {
        u16le(&mut out, z.sample);
        out.extend_from_slice(&[
            z.program,
            z.drum_key,
            z.key_range[0],
            z.key_range[1],
            z.velocity_range[0],
            z.velocity_range[1],
            z.root_key,
            0,
        ]);
        u16le(&mut out, z.cents as u16);
        u16le(&mut out, z.gain);
        u16le(&mut out, z.pan as u16);
        u32le(&mut out, z.attack_ms);
        u32le(&mut out, z.decay_ms);
        u16le(&mut out, z.sustain);
        u16le(&mut out, 0);
        u32le(&mut out, z.release_ms);
    }
    out.resize(data_offset, 0);
    for s in &bank.samples {
        out.extend_from_slice(&s.bytes);
    }
    out
}

pub fn cook(
    root: &Path,
    package: &assets::Package,
    index: &assets::Index,
) -> Result<Cooked, String> {
    cook_cancelled(root, package, index, &std::sync::atomic::AtomicBool::new(false))
}
pub fn cook_cancelled(root: &Path, package: &assets::Package, index: &assets::Index,
    cancelled: &std::sync::atomic::AtomicBool) -> Result<Cooked, String> {
    if cancelled.load(std::sync::atomic::Ordering::Relaxed) { return Err("PSX sequence cook cancelled".into()); }
    let settings = package.meta.settings.sequence()?;
    let ir = crate::sequence::decode_source(&package.source,settings)?;
    let record = crate::sequence::resolve_bank(root, settings, index)?;
    let library = record.meta.settings.sound_bank()?.library.is_some();
    let bank = if library {
        crate::psx_library_asset::cook(root, package, record, &ir, cancelled)?
    } else {
        if crate::psx_music_settings::Recipe::from_settings(settings)?.effects != crate::psx_music_settings::Effects::Dry {
            return Err("The Room reverb profile requires an instrument library SoundBank; EPSB v1 retains its dry playback contract".into());
        }
        if !settings.instrument_mappings.is_empty() {
            return Err("Explicit library mappings require an instrument library SoundBank; portable v1 banks use their saved zones".into());
        }
        record.meta.settings.sound_bank()?.validate_sequence(&ir, index)?;
        bank(root, record, index)?
    };
    let (events, payload) = if library { library_payload(&ir,settings,bank.id)? } else { sequence_payload(&ir,settings,bank.id)? };
    let elided_noop_events=if library { sequence_stream::events(&ir,settings)?.len()-events.len() } else { 0 };
    let (prepared_start_states,prepared_start_references) = if library { crate::instrument_preview::prepared_count(&events,ir.ppqn,&bank.payload)? } else { (0,0) };
    let payload_version = u16::from_le_bytes([payload[4], payload[5]]);
    let mut inputs = bank.inputs.clone();
    inputs.insert(
        format!("asset:{}", package.meta.id),
        assets::cache_key(&package.meta),
    );
    if settings.sound_bank.is_none() {
        inputs.insert("default-sound-bank".into(), record.meta.id.to_string());
    }
    let sample_bytes = bank.samples.iter().map(|s| s.bytes.len()).sum();
    let report = Report {
        target: "psx",
        profile: if library { crate::psx_music_settings::PROFILE } else if payload_version == 1 { PROFILE } else { MUSICAL_PROFILE },
        resolved_mode: "Resident sequence",
        bank_mode: "Resident SoundBank",
        payload_version,
        endianness: "little",
        sequence_bytes: payload.len(),
        bank_bytes: bank.payload.len(),
        sample_main_ram_bytes: sample_bytes,
        prepared_start_states,
        prepared_start_references,
        elided_noop_events,
        spu_ram_bytes: sample_bytes,
        package_bytes: package.bytes()?.len() as u64 + bank.package_bytes,
        voice_limit: settings.voices(),
        peak_polyphony: ir.peak_polyphony,
        warnings: bank.warnings.clone(),
    };
    let cache = root.join(".epok/imported").join(identity(&inputs));
    if cancelled.load(std::sync::atomic::Ordering::Relaxed) { return Err("PSX sequence cook cancelled".into()); }
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    assets::replace_cache(&cache.join("sequence.epsq"), &payload)?;
    let summary = root
        .join(".epok/imported")
        .join(assets::cache_key(&package.meta));
    std::fs::create_dir_all(&summary).map_err(|e| e.to_string())?;
    let report_bytes = serde_json::to_vec(&report).map_err(|e| e.to_string())?;
    assets::replace_cache(&cache.join("Import.epokcache"), &report_bytes)?;
    let summary_bytes = serde_json::to_vec(&serde_json::json!({"report":report,"inputs":inputs}))
        .map_err(|e| e.to_string())?;
    assets::replace_cache(&summary.join("Sequence.epokcache"), &summary_bytes)?;
    Ok(Cooked {
        payload,
        bank,
        report,
        inputs,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn whole_loop_setup_moves_only_the_initial_snapshot() {
        use crate::sequence_stream::Event;
        let e=|tick,op,a,b,value|Event{tick,op,channel:0,a,b,value};
        let source=vec![e(0,6,0,0,0),e(0,5,0,0,400000),e(0,9,0,0,1200),
            e(0,3,64,127,0),e(0,0,60,100,0),e(0,3,7,80,0),e(1,1,60,0,0),e(2,7,0,0,0)];
        let mut cooked=source.clone();
        assert_eq!(super::hoist_whole_loop_setup(&mut cooked),3);
        assert_eq!(cooked[3],source[0]);
        assert_eq!(cooked.iter().filter(|e|e.op!=6).collect::<Vec<_>>(),source.iter().filter(|e|e.op!=6).collect::<Vec<_>>());
        let mut later=vec![e(0,6,0,0,0),e(1,2,1,0,0)];
        assert_eq!(super::hoist_whole_loop_setup(&mut later),0);
        let mut intro=vec![e(0,0,60,100,0),e(1,6,0,0,0),e(1,3,7,80,0)];
        let original=intro.clone();
        assert_eq!(super::hoist_whole_loop_setup(&mut intro),0);assert_eq!(intro,original);
    }
    #[test]
    fn library_compaction_retains_notes_resets_explicit_sends_and_changes() {
        use crate::sequence_stream::Event;
        let e=|tick,op,a,b,value|Event{tick,op,channel:0,a,b,value};
        let mut events=vec![e(0,6,0,0,0),e(0,9,0,0,200),e(0,9,0,0,1200),
            e(0,3,7,100,0),e(0,3,91,0,0),e(0,3,93,0,0),e(0,0,60,100,0),
            e(1,4,0,0,8192),e(1,3,91,0,0),e(1,3,64,127,0),e(1,3,64,100,0),
            e(2,3,121,0,0),e(2,3,64,127,0),e(2,1,60,0,0),e(3,7,0,0,0)];
        let notes=events.iter().copied().filter(|e|e.op<=1).collect::<Vec<_>>();
        super::compact_noop_controls(&mut events);
        assert_eq!(events.iter().filter(|e|e.op<=1).copied().collect::<Vec<_>>(),notes);
        assert_eq!(events.len(),9);
        assert!(events.contains(&e(0,9,0,0,1200)) && events.contains(&e(0,3,91,0,0)));
        assert!(events.contains(&e(2,3,121,0,0)) && events.contains(&e(2,3,64,127,0)));
        assert_eq!(events.last(),Some(&e(3,7,0,0,0)));
    }
    use super::*;
    #[test]
    fn psx_sequence_and_sfx_share_a_hard_budget_without_residency_fallback() {
        let root = crate::workspace::tests::temp("psx-bank-budget");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let tone = audio_import::test_wav();
        let mut long = tone[..44].to_vec();
        for _ in 0..300 {
            long.extend_from_slice(&tone[44..]);
        }
        let bytes = long.len() as u32;
        long[4..8].copy_from_slice(&(bytes - 8).to_le_bytes());
        long[40..44].copy_from_slice(&(bytes - 44).to_le_bytes());
        std::fs::write(root.join("assets/long.wav"), long).unwrap();
        let sample = assets::commit(
            assets::prepare(
                &root,
                "assets/long.wav",
                "assets/long.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let bank = crate::sound_bank::create(
            &root,
            "assets/bank.epokasset",
            crate::sound_bank::Settings {
                programs: vec![crate::sound_bank::Program {
                    program: 0,
                    drum_key: None,
                    zones: vec![crate::sound_bank::Zone::new(sample)],
                    extra: Default::default(),
                }],
                ..Default::default()
            },
        )
        .unwrap();
        std::fs::write(root.join("assets/song.mid"), crate::midi::fixture()).unwrap();
        let song = assets::commit(
            crate::sequence::prepare(
                &root,
                "assets/song.mid",
                "assets/song.epokasset",
                crate::sequence::Settings {
                    sound_bank: Some(bank),
                    ..Default::default()
                },
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let before = std::fs::read(&index.resolve(sample).unwrap().path).unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.entities[0].audio = Some(crate::audio::AudioSource {
            clip: Some(sample),
            ..Default::default()
        });
        scene.entities[1].audio = Some(crate::audio::AudioSource {
            clip: Some(song),
            ..Default::default()
        });
        let error = crate::audio::stage(&root, &scene, &root.join("build"), &index).unwrap_err();
        assert!(error.contains("PSX budget is 520192"), "{error}");
        assert_eq!(
            before,
            std::fs::read(&index.resolve(sample).unwrap().path).unwrap()
        );
        assert_eq!(
            index
                .resolve(sample)
                .unwrap()
                .meta
                .settings
                .audio()
                .unwrap()
                .load_mode,
            audio_import::LoadMode::Resident
        );
    }
    #[test]
    fn psx_sequence_staging_tracks_bank_samples_and_target_preview() {
        let root = crate::workspace::tests::temp("psx-sequence-stage");
        let (sample, bank) = crate::sequence_preview::tests::fixture(&root);
        std::fs::write(root.join("assets/song.mid"), crate::midi::fixture()).unwrap();
        let settings = crate::sequence::Settings {
            sound_bank: Some(bank),
            ..Default::default()
        };
        let id = assets::commit(
            crate::sequence::prepare(
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
        let mut scene = crate::scene::Scene::default();
        scene.entities[0].audio = Some(crate::audio::AudioSource {
            clip: Some(id),
            ..Default::default()
        });
        let output = crate::audio::stage(&root, &scene, &root.join("build"), &index).unwrap();
        let header = std::fs::read_to_string(root.join("build/audio-bank.hh")).unwrap();
        assert!(header.contains("#define EPOK_HAS_SEQUENCES 1"));
        assert!(header.contains("&sequence_asset_0"));
        let inputs = &output
            .iter()
            .find(|o| o.path == "audio-bank.hh")
            .unwrap()
            .inputs;
        assert!(inputs.contains_key(&format!("asset:{sample}")));
        assert!(inputs.contains_key(&format!("asset:{bank}")));
        assert_eq!(
            inputs[&format!("audio-cook:{id}")],
            cook_key(&root, &index.resolve(id).unwrap().meta).unwrap()
        );
        let (target, _) = crate::sequence_preview::render_target(
            &root,
            &crate::midi::fixture(),
            &settings,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        assert!(target.samples.iter().any(|v| v.abs() > 1000));
        assert!(target.report.unwrap().contains("does not emulate"));
        let report =
            std::fs::read_to_string(root.join("build/audio/sequence-report.json")).unwrap();
        assert!(report.contains("1344"));
        let first = inputs[&format!("audio-cook:{id}")].clone();
        let mut package = assets::Package::load(&index.resolve(sample).unwrap().path).unwrap();
        let crate::import_settings::Settings::Audio(s) = &mut package.meta.settings else {
            panic!()
        };
        s.normalize = !s.normalize;
        std::fs::write(
            &index.resolve(sample).unwrap().path,
            package.bytes().unwrap(),
        )
        .unwrap();
        assert_ne!(
            first,
            cook_key(&root, &index.resolve(id).unwrap().meta).unwrap()
        );
    }
    #[test]
    fn psx_sequence_payload_is_versioned_bounded_and_little_endian() {
        let ir = crate::midi::parse(&crate::midi::fixture()).unwrap();
        let settings = crate::sequence::Settings::default();
        let (events, bytes) = sequence_payload(&ir, &settings, Uuid::from_u128(1)).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(bytes.len(), 76);
        assert_eq!(&bytes[..12], b"EPSQ\x01\0\x28\0\x60\0\x10\0");
        assert_eq!(&bytes[40..52], &[0, 0, 0, 0, 0, 0, 60, 100, 0, 0, 0, 0]);
        assert_eq!(&bytes[52..64], &[96, 0, 0, 0, 1, 0, 60, 0, 0, 0, 0, 0]);
        assert_eq!(&bytes[64..], &[96, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0]);
        assert!(
            sequence_payload(
                &ir,
                &crate::sequence::Settings {
                    voice_limit: Some(25),
                    ..settings
                },
                Uuid::from_u128(1)
            )
            .unwrap_err()
            .contains("24")
        );
    }
    #[test]
    fn midi_extended_semantics_require_epsq_v2_and_bank_identity_never_falls_back() {
        use crate::sequence_ir::{Event, EventKind, SequenceIr};
        let ir = SequenceIr::analyze(480, vec![
            Event { tick: 0, track: 0, order: 0, kind: EventKind::Parameter { channel: 0, parameter: 0, value: 1200 } },
            Event { tick: 0, track: 0, order: 1, kind: EventKind::BankProgram { channel: 0, bank: 130, program: 5 } },
            Event { tick: 0, track: 0, order: 2, kind: EventKind::NoteOn { channel: 0, key: 60, velocity: 100 } },
            Event { tick: 480, track: 0, order: 3, kind: EventKind::NoteOff { channel: 0, key: 60 } },
            Event { tick: 480, track: 0, order: 4, kind: EventKind::EndTrack },
        ], vec![]).unwrap();
        let settings = crate::sequence::Settings::default();
        let (events, payload) = sequence_payload(&ir, &settings, Uuid::from_u128(1)).unwrap();
        assert_eq!(&payload[..8], b"EPSQ\x02\0\x28\0");
        assert_eq!((events[0].op, events[0].a, events[0].value), (9, 0, 1200));
        assert_eq!((events[1].op, events[1].a, events[1].value), (10, 5, 130));
        let error = crate::sound_bank::Settings::default().validate_sequence(&ir, &assets::Index::default()).unwrap_err();
        assert!(error.contains("bank MSB 1/LSB 2"), "{error}");
        assert!(error.contains("bank 0 only"), "{error}");
    }
    #[test]
    fn psx_bank_cooks_original_samples_deduplicates_and_resolves_block_loops() {
        let root = crate::workspace::tests::temp("psx-sequence-bank");
        let (_, id) = crate::sequence_preview::tests::fixture(&root);
        let index = assets::scan(&root, &mut Default::default());
        let cooked = bank(&root, index.resolve(id).unwrap(), &index).unwrap();
        assert_eq!(cooked.samples.len(), 1);
        assert_eq!(cooked.samples[0].bytes.len(), 1344);
        assert_eq!(cooked.samples[0].loop_region, Some([28, 2240]));
        assert_eq!(cooked.payload.len(), 1472);
        assert_eq!(&cooked.payload[..8], b"EPSB\x01\0\x20\0");
        assert_eq!(cooked.samples[0].bytes[16] >> 4, 0);
        assert_eq!(cooked.samples[0].bytes[17] & 4, 4);
        assert_eq!(
            audio_import::decode_adpcm(&cooked.samples[0].bytes)
                .unwrap()
                .len(),
            2240
        );
        assert!(
            cooked
                .warnings
                .iter()
                .any(|w| w.contains("resolved outward"))
        );
        assert_eq!(
            cooked.payload,
            bank(&root, index.resolve(id).unwrap(), &index)
                .unwrap()
                .payload
        );
        let record = index.resolve(id).unwrap();
        assert_eq!(
            identity(&cooked.inputs),
            cook_key(&root, &record.meta).unwrap()
        );
        let cache = root.join(".epok/imported").join(identity(&cooked.inputs));
        std::fs::write(cache.join("bank.epokcache"), "corrupt").unwrap();
        assert_eq!(cooked.payload, bank(&root, record, &index).unwrap().payload);
    }
    #[test]
    fn psx_arbitrary_loop_entry_uses_independent_filter_and_excludes_tail() {
        let settings = audio_import::Settings::default();
        let (cooked, _) = sample(&audio_import::test_wav(), &settings, Some([56, 280])).unwrap();
        assert_eq!(cooked.loop_region, Some([84, 308]));
        assert_eq!(cooked.bytes[48] >> 4, 0);
        assert_eq!(cooked.bytes[49], 4);
        assert_eq!(cooked.bytes[10 * 16 + 1], 3);
        assert_eq!(
            audio_import::decode_adpcm(&cooked.bytes).unwrap().len(),
            308
        );
        assert!(sample(&audio_import::test_wav(), &settings, Some([0, 2206])).is_err());
    }
}
