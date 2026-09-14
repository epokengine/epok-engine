//! Asset/cache boundary for reduced library banks. Authoritative assets are read
//! and revision-checked; only complete derivatives are published to the cache.
use crate::{
    assets, psx_library,
    psx_sequence::{Bank, LibraryMetadata, Sample},
};
use std::{
    collections::BTreeMap,
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEnvelope {
    checksum: String,
    bank: Bank,
}
const MAX_CACHE_BYTES: u64 = 4 * 1024 * 1024;

/// The observer and cooker must derive the same key from authoritative metadata.
/// Package loading verifies these source hashes against the snapshot bytes.
pub fn inputs(
    sequence: &assets::Metadata,
    library: &assets::Metadata,
) -> Result<BTreeMap<String, String>, String> {
    let settings = sequence.settings.sequence()?;
    let recipe = crate::psx_music_settings::Recipe::from_settings(settings)?;
    let scope = format!("library:{}", sequence.id);
    let mut inputs = BTreeMap::from([
        (
            format!("asset:{}", sequence.id),
            assets::cache_key(sequence),
        ),
        (format!("asset:{}", library.id), assets::cache_key(library)),
        (format!("{scope}:source"), library.source_hash.clone()),
        (
            format!("{scope}:sequence-selection"),
            sequence.source_hash.clone(),
        ),
        (
            format!("{scope}:profile"),
            crate::psx_music_settings::PROFILE.into(),
        ),
        (
            format!("{scope}:recipe"),
            serde_json::to_string(&recipe).map_err(|e| e.to_string())?,
        ),
    ]);
    if settings.sound_bank.is_none() {
        inputs.insert("default-sound-bank".into(), library.id.to_string());
    }
    Ok(inputs)
}

fn read_cache(path: &Path) -> Option<CacheEnvelope> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_CACHE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_CACHE_BYTES {
        return None;
    }
    let cached: CacheEnvelope = serde_json::from_slice(&bytes).ok()?;
    (cached.checksum == assets::hash(&serde_json::to_vec(&cached.bank).ok()?)).then_some(cached)
}

/// Bounded streaming verification avoids allocating another complete library
/// source while its decoded PCM and cooked samples are already resident.
pub fn verify_record(record: &assets::Record, cancelled: &AtomicBool) -> Result<u64, String> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(&record.path).map_err(|e| e.to_string())?;
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 65536];
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err("PSX library conversion cancelled".into());
        }
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > assets::MAX_SOUNDFONT_SOURCE as u64 + 65536 + 16 {
            return Err("Asset exceeds the bounded library snapshot extent".into());
        }
        digest.update(&buffer[..n]);
    }
    if format!("{:x}", digest.finalize()) != record.revision {
        return Err("Instrument library or sequence changed during conversion; retry with its current revision".into());
    }
    Ok(bytes)
}

pub fn cook(
    root: &Path,
    sequence: &assets::Package,
    record: &assets::Record,
    ir: &crate::sequence_ir::SequenceIr,
    cancelled: &AtomicBool,
) -> Result<Bank, String> {
    let settings = sequence.meta.settings.sequence()?;
    settings.validate_playback(ir)?;
    let package = assets::Package::load(&record.path)?;
    let bank_settings = package.meta.settings.sound_bank()?;
    bank_settings.validate()?;
    if bank_settings.library.is_none() {
        return Err("Library conversion requires a source instrument library".into());
    }
    if bank_settings.load_mode == crate::audio_import::LoadMode::Stream {
        return Err(
            "The PSX library bank requires Resident/Auto; no bank streaming profile is implemented"
                .into(),
        );
    }
    let verify = || -> Result<(), String> {
        if cancelled.load(Ordering::Relaxed) {
            return Err("PSX library conversion cancelled".into());
        }
        if package.meta.id != record.meta.id
            || assets::cache_key(&package.meta) != assets::cache_key(&record.meta)
        {
            return Err(
                "Instrument library changed during conversion; retry with its current revision"
                    .into(),
            );
        }
        verify_record(record, cancelled)?;
        Ok(())
    };
    verify()?;
    let recipe = crate::psx_music_settings::Recipe::from_settings(settings)?;
    let inputs = inputs(&sequence.meta, &package.meta)?;
    let key = crate::psx_sequence::identity(&inputs);
    let id = derived_id(&key);
    let cache = root.join(".epok/imported").join(&key);
    let cached = cache.join("library.epokcache");
    if let Some(CacheEnvelope { mut bank, .. }) = read_cache(&cached)
        && bank.id == id
        && bank.inputs == inputs
        && bank.zones.is_empty()
        && bank.samples.len() <= psx_library::MAX_SAMPLES
        && let Some(metadata) = &bank.library
        && metadata.report.recipe == recipe
        && metadata.report.profile == crate::psx_music_settings::PROFILE
        && metadata.zones.len() <= psx_library::MAX_ZONES
        && let Ok(cooked) = from_cached(&bank)
        && ensure_budget(&cooked).is_ok()
        && let Ok(payload) = crate::psx_library_wire::encode(&cooked)
    {
        bank.payload = payload;
        verify()?;
        return Ok(bank);
    }
    let prepared = psx_library::prepare_selection(
        &package.source,
        ir,
        &settings.instrument_mappings,
        recipe.selection,
        cancelled,
    )?;
    let cooked = psx_library::cook(&prepared, &recipe, cancelled)?;
    ensure_budget(&cooked)?;
    let payload = crate::psx_library_wire::encode(&cooked)?;
    verify()?;
    let warnings = cooked
        .report
        .adaptations
        .iter()
        .map(|a| format!("{} {:?}: {}", a.code, a.region, a.detail))
        .collect();
    let bank = Bank {
        id,
        inputs,
        payload,
        zones: vec![],
        samples: cooked
            .samples
            .into_iter()
            .map(|s| Sample {
                bytes: s.bytes,
                rate: s.rate,
                frames: s.frames,
                loop_region: s.loop_region,
            })
            .collect(),
        library: Some(LibraryMetadata {
            zones: cooked.zones,
            report: cooked.report,
        }),
        package_bytes: std::fs::metadata(&record.path)
            .map_err(|e| e.to_string())?
            .len(),
        warnings,
    };
    let checksum = assets::hash(&serde_json::to_vec(&bank).map_err(|e| e.to_string())?);
    let envelope = CacheEnvelope { checksum, bank };
    let bytes = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    // One atomic commit contains metadata and its checksum. A cancelled or
    // interrupted writer cannot expose a partially committed three-file cache.
    verify()?;
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    if bytes.len() as u64 <= MAX_CACHE_BYTES {
        assets::replace_cache(&cache.join("library.epokcache"), &bytes)?;
    }
    // Atomic commit wins a simultaneous late cancellation. The completed cache
    // describes this immutable snapshot; a UI worker still discards stale results.
    Ok(envelope.bank)
}

fn ensure_budget(cooked: &psx_library::Cooked) -> Result<(), String> {
    let bytes = cooked
        .samples
        .iter()
        .try_fold(0_usize, |sum, s| sum.checked_add(s.bytes.len()))
        .ok_or("Library sample sizes overflow")?;
    if bytes != cooked.report.sample_spu_bytes
        || bytes > cooked.report.recipe.available_bytes() as usize
    {
        return Err(format!(
            "PSX music samples require {bytes} bytes; available {} after the configured other-resident reservation. Choose a smaller recipe or explicitly optimize to budget; the bank remains Resident",
            cooked.report.recipe.available_bytes()
        ));
    }
    Ok(())
}

pub fn from_cached(bank: &Bank) -> Result<psx_library::Cooked, String> {
    let metadata = bank
        .library
        .as_ref()
        .ok_or("Cached bank is not a library derivative")?;
    Ok(psx_library::Cooked {
        zones: metadata.zones.clone(),
        report: metadata.report.clone(),
        samples: bank
            .samples
            .iter()
            .map(|s| psx_library::Sample {
                source_sample: 0,
                bytes: s.bytes.clone(),
                rate: s.rate,
                frames: s.frames,
                loop_region: s.loop_region,
                squared_error: 0,
                input_frames: 0,
                improved_encoder: false,
            })
            .collect(),
    })
}

/// UUID v8 is an internal derivative identifier, not the library's authoring UUID.
fn derived_id(key: &str) -> uuid::Uuid {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(key.as_bytes());
    let mut bytes: [u8; 16] = digest[..16].try_into().unwrap();
    bytes[6] = bytes[6] & 15 | 0x80;
    bytes[8] = bytes[8] & 63 | 0x80;
    uuid::Uuid::from_bytes(bytes)
}
