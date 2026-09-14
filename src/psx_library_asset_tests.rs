//! End-to-end asset/cache coverage for song-specific SoundFont PSX banks.
//!
//! Registered by the crate root only under `cfg(test)`: this keeps production
//! dependency and staging surfaces unchanged.
use crate::{
    assets, psx_library_asset,
    psx_music_settings::{Preset, Recipe},
    sequence,
};
use std::{fs, path::Path, sync::atomic::AtomicBool};
use uuid::Uuid;

struct Fixture {
    root: std::path::PathBuf,
    bank: Uuid,
    song: Uuid,
}

fn fixture(name: &str) -> Fixture {
    let root = crate::workspace::tests::temp(name);
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::write(root.join("assets/library.sf2"), crate::sf2::fixture()).unwrap();
    let mut bank_settings = crate::sound_bank::Settings::default();
    bank_settings.extra.insert(
        "future_bank_setting".into(),
        serde_json::json!({"keep": 17}),
    );
    bank_settings
        .target_overrides
        .insert("future_target".into(), serde_json::json!({"keep": true}));
    let bank = assets::commit(
        crate::soundfont_asset::prepare(
            &root,
            "assets/library.sf2",
            "assets/library.epokasset",
            bank_settings,
            None,
            false,
        )
        .unwrap(),
    )
    .unwrap();
    let song = commit_song(&root, "song", crate::midi::fixture(), bank, None, None);
    Fixture { root, bank, song }
}

fn commit_song(
    root: &Path,
    name: &str,
    source: Vec<u8>,
    bank: Uuid,
    rate: Option<u32>,
    existing: Option<&assets::Record>,
) -> Uuid {
    let source_path = format!("assets/{name}.mid");
    let asset_path = format!("assets/{name}.epokasset");
    if existing.is_none() {
        fs::write(root.join(&source_path), source).unwrap();
    }
    let mut settings = sequence::Settings {
        sound_bank: Some(bank),
        ..Default::default()
    };
    settings.extra.insert(
        "future_sequence_setting".into(),
        serde_json::json!(["keep", 29]),
    );
    if let Some(rate) = rate {
        let mut recipe = Recipe::default();
        recipe.preset = Preset::Custom;
        recipe.max_sample_rate = rate;
        recipe.store(&mut settings).unwrap();
    }
    assets::commit(
        sequence::prepare(
            root,
            if existing.is_some() {
                "assets/missing.mid"
            } else {
                &source_path
            },
            &asset_path,
            settings,
            existing,
            existing.is_some(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn replace_recipe(root: &Path, existing: &assets::Record, bank: Uuid, recipe: Recipe) -> Uuid {
    let mut settings = sequence::Settings {
        sound_bank: Some(bank),
        ..Default::default()
    };
    settings.extra.insert(
        "future_sequence_setting".into(),
        serde_json::json!(["keep", 29]),
    );
    recipe.store(&mut settings).unwrap();
    assets::commit(
        sequence::prepare(
            root,
            "assets/missing.mid",
            "assets/song.epokasset",
            settings,
            Some(existing),
            true,
        )
        .unwrap(),
    )
    .unwrap()
}

fn cooked(
    root: &Path,
    song: Uuid,
    bank: Uuid,
    cancelled: &AtomicBool,
) -> Result<crate::psx_sequence::Bank, String> {
    let index = assets::scan(root, &mut Default::default());
    let sequence_record = index.resolve(song)?;
    let bank_record = index.resolve(bank)?;
    let package = assets::Package::load(&sequence_record.path)?;
    let settings = package.meta.settings.sequence()?;
    let ir = sequence::decode_source(&package.source, settings)?;
    psx_library_asset::cook(root, &package, bank_record, &ir, cancelled)
}

#[test]
fn library_cook_is_snapshot_stable_and_rebuilds_corrupt_cache() {
    let fixture = fixture("psx-library-asset-cold-warm");
    let before_index = assets::scan(&fixture.root, &mut Default::default());
    let bank_record = before_index.resolve(fixture.bank).unwrap().clone();
    let before = assets::Package::load(&bank_record.path).unwrap();
    let cold = cooked(
        &fixture.root,
        fixture.song,
        fixture.bank,
        &AtomicBool::new(false),
    )
    .unwrap();
    let warm = cooked(
        &fixture.root,
        fixture.song,
        fixture.bank,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(cold.id, warm.id);
    assert_eq!(cold.payload, warm.payload);
    assert_eq!(cold.inputs, warm.inputs);
    assert_eq!(
        crate::psx_sequence::identity(&cold.inputs),
        crate::psx_sequence::cook_key(
            &fixture.root,
            &before_index.resolve(fixture.song).unwrap().meta
        )
        .unwrap(),
        "staged library cook identity must survive dependency observation"
    );
    assert_eq!(
        cold.library.as_ref().unwrap().report.sample_spu_bytes,
        warm.library.as_ref().unwrap().report.sample_spu_bytes
    );

    let after = assets::Package::load(&bank_record.path).unwrap();
    assert_eq!(after.meta.id, before.meta.id);
    assert_eq!(after.source, before.source);
    assert_eq!(after.meta.source_hash, before.meta.source_hash);
    let settings = after.meta.settings.sound_bank().unwrap();
    assert_eq!(
        settings.extra["future_bank_setting"],
        serde_json::json!({"keep": 17})
    );
    assert_eq!(
        settings.target_overrides["future_target"],
        serde_json::json!({"keep": true})
    );

    let cache = fixture
        .root
        .join(".epok/imported")
        .join(crate::psx_sequence::identity(&cold.inputs));
    fs::write(cache.join("library.epokcache"), b"corrupt").unwrap();
    let rebuilt = cooked(
        &fixture.root,
        fixture.song,
        fixture.bank,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(rebuilt.id, cold.id);
    assert_eq!(rebuilt.payload, cold.payload);
    let cached = fs::read(cache.join("library.epokcache")).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&cached).unwrap();
    let stored: crate::psx_sequence::Bank =
        serde_json::from_value(envelope["bank"].clone()).unwrap();
    assert_eq!(
        envelope["checksum"],
        assets::hash(&serde_json::to_vec(&stored).unwrap())
    );
    // A self-consistent but semantically corrupt cache also rebuilds safely.
    let mut bad = stored;
    bad.samples[0].frames = u32::MAX;
    let checksum = assets::hash(&serde_json::to_vec(&bad).unwrap());
    fs::write(
        cache.join("library.epokcache"),
        serde_json::to_vec(&serde_json::json!({"checksum":checksum,"bank":bad})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        cooked(
            &fixture.root,
            fixture.song,
            fixture.bank,
            &AtomicBool::new(false)
        )
        .unwrap()
        .payload,
        cold.payload
    );
}

#[test]
fn sequence_selection_and_recipe_are_part_of_library_derivative_identity() {
    let fixture = fixture("psx-library-asset-identity");
    let first = cooked(
        &fixture.root,
        fixture.song,
        fixture.bank,
        &AtomicBool::new(false),
    )
    .unwrap();
    let mut second_midi = crate::midi::fixture();
    let key = second_midi.iter().position(|byte| *byte == 60).unwrap();
    second_midi[key] = 61;
    let second_song = commit_song(
        &fixture.root,
        "different-key",
        second_midi,
        fixture.bank,
        None,
        None,
    );
    let second = cooked(
        &fixture.root,
        second_song,
        fixture.bank,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_ne!(first.id, second.id);
    assert_ne!(
        first.inputs[&format!("library:{}:sequence-selection", fixture.song)],
        second.inputs[&format!("library:{second_song}:sequence-selection")]
    );
    assert_eq!(first.library.as_ref().unwrap().zones.len(), 1);
    assert_eq!(second.library.as_ref().unwrap().zones.len(), 1);

    let index = assets::scan(&fixture.root, &mut Default::default());
    let old = index.resolve(fixture.song).unwrap().clone();
    assert_eq!(
        commit_song(
            &fixture.root,
            "song",
            Vec::new(),
            fixture.bank,
            Some(11_025),
            Some(&old)
        ),
        fixture.song
    );
    let changed = cooked(
        &fixture.root,
        fixture.song,
        fixture.bank,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_ne!(changed.id, first.id);
    assert_ne!(
        changed.inputs[&format!("library:{}:recipe", fixture.song)],
        first.inputs[&format!("library:{}:recipe", fixture.song)]
    );
    let sequence_package = assets::Package::load(&old.path).unwrap();
    assert_eq!(
        sequence_package.meta.settings.sequence().unwrap().extra["future_sequence_setting"],
        serde_json::json!(["keep", 29])
    );
}

#[test]
fn failed_budget_cancellation_and_stale_library_records_publish_nothing() {
    let fixture = fixture("psx-library-asset-failures");
    let index = assets::scan(&fixture.root, &mut Default::default());
    let original = index.resolve(fixture.song).unwrap().clone();
    let mut impossible = Recipe::default();
    impossible.preset = Preset::Custom;
    impossible.bank_budget_bytes = 1;
    assert_eq!(
        replace_recipe(&fixture.root, &original, fixture.bank, impossible),
        fixture.song
    );
    let budget_error = match cooked(
        &fixture.root,
        fixture.song,
        fixture.bank,
        &AtomicBool::new(false),
    ) {
        Ok(_) => panic!("an impossible resident budget must not publish a bank"),
        Err(error) => error,
    };
    assert!(budget_error.contains("require") || budget_error.contains("budget"));
    assert!(!fixture.root.join(".epok/imported").exists());

    let cancelled = AtomicBool::new(true);
    let cancel_error = match cooked(&fixture.root, fixture.song, fixture.bank, &cancelled) {
        Ok(_) => panic!("a cancelled cook must not publish a bank"),
        Err(error) => error,
    };
    assert!(cancel_error.contains("cancelled"));

    let index = assets::scan(&fixture.root, &mut Default::default());
    let package = assets::Package::load(&index.resolve(fixture.song).unwrap().path).unwrap();
    let settings = package.meta.settings.sequence().unwrap();
    let ir = sequence::decode_source(&package.source, settings).unwrap();
    let mut stale = index.resolve(fixture.bank).unwrap().clone();
    stale.revision = "stale-cas-revision".into();
    let stale_error = match psx_library_asset::cook(
        &fixture.root,
        &package,
        &stale,
        &ir,
        &AtomicBool::new(false),
    ) {
        Ok(_) => panic!("a stale SoundBank record must not publish a bank"),
        Err(error) => error,
    };
    assert!(stale_error.contains("changed during conversion"));
}
