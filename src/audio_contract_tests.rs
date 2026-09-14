//! Byte-level legacy PSX oracle, captured before the portable settings migration.
use crate::{assets, audio_import};

#[test]
fn audio_authoring_valid_target_error_and_tool_identity() {
    let root = crate::workspace::tests::temp("audio-cook-identity");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(root.join("assets/tone.wav"), audio_import::test_wav()).unwrap();
    let settings = audio_import::Settings {
        channels: 2,
        ..Default::default()
    };
    let candidate = assets::prepare(
        &root,
        "assets/tone.wav",
        "assets/tone.epokasset",
        settings,
        None,
        false,
    )
    .unwrap();
    assert!(
        assets::derived(&root, &candidate.package)
            .unwrap_err()
            .contains("mono")
    );
    assets::commit(candidate).unwrap();
    assert!(!root.join(".epok/imported").exists());
    let mut package = assets::Package::load(&root.join("assets/tone.epokasset")).unwrap();
    package.meta.settings = crate::import_settings::Settings::Audio(audio_import::Settings {
        load_mode: audio_import::LoadMode::Stream,
        sample_rate: 37800,
        ..Default::default()
    });
    let tool = root.join(".tools/psxavenc/bin/psxavenc.exe");
    std::fs::write(root.join("Local.epokconfig"), b"{}").unwrap();
    std::fs::create_dir_all(tool.parent().unwrap()).unwrap();
    std::fs::write(&tool, b"encoder revision one").unwrap();
    assert_eq!(crate::disc::tool(&root, "psxavenc").unwrap(), tool);
    let before = assets::cook_key(&root, &package.meta).unwrap();
    let authoring = assets::cache_key(&package.meta);
    std::fs::write(tool, b"encoder revision two").unwrap();
    assert_ne!(before, assets::cook_key(&root, &package.meta).unwrap());
    assert_eq!(authoring, assets::cache_key(&package.meta));
}

#[test]
#[ignore = "Owns an ImGui context; run serially to exercise real import controls"]
fn audio_import_dialog_clicks_publish_package() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [1440., 1000.];
    context.io_mut().delta_time = 1. / 60.;
    context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    context.fonts().build_rgba32_texture();
    crate::asset_ui::interaction::verify(&mut context);
}

#[test]
#[ignore = "Owns an ImGui context; run serially for real MIDI bank selection and import controls"]
fn midi_import_dialog_assigns_bank_and_publishes_sequence() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [1440., 1000.];
    context.io_mut().delta_time = 1. / 60.;
    context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    context.fonts().build_rgba32_texture();
    crate::asset_ui::interaction::verify_sequence(&mut context);
}

#[test]
fn audio_schema_migrates_both_legacy_profiles_and_preserves_unknowns() {
    use audio_import::{AudioRole, LoadMode, Settings};
    for (usage, role, mode, rate, channels) in [
        ("Sfx", AudioRole::Sfx, LoadMode::Resident, 22050, 1),
        ("Music", AudioRole::Music, LoadMode::Stream, 37800, 2),
    ] {
        let value = serde_json::json!({
            "usage":usage,"sample_rate":rate,"channels":channels,"normalize":true,"looping":true,
            "future_option":{"value":17},"target_overrides":{"future":{"opaque":[1,2]},"psx":{"future_rate_policy":7}}
        });
        let settings: Settings = serde_json::from_value(value).unwrap();
        assert_eq!(
            (settings.role, settings.load_mode, settings.rate()),
            (role, mode, rate)
        );
        settings.validate_psx().unwrap();
        let written = serde_json::to_value(&settings).unwrap();
        assert_eq!(written["target_overrides"]["psx"]["sample_rate"], rate);
        assert_eq!(written["target_overrides"]["psx"]["future_rate_policy"], 7);
        assert_eq!(written["future_option"]["value"], 17);
        assert_eq!(
            written["target_overrides"]["future"]["opaque"],
            serde_json::json!([1, 2])
        );
        assert!(written.get("usage").is_none() && written.get("sample_rate").is_none());
        let restored: Settings = serde_json::from_value(written).unwrap();
        assert_eq!(settings, restored);
    }
    for invalid in [
        serde_json::json!({"usage":null,"sample_rate":22050}),
        serde_json::json!({"schema_version":99,"role":"Music","load_mode":"Resident"}),
        serde_json::json!({"role":"Music","load_mode":"Resident","sample_rate":22050}),
        serde_json::json!({"type":"Unknown","sample_rate":22050}),
    ] {
        assert!(serde_json::from_value::<crate::import_settings::Settings>(invalid).is_err());
    }
    let envelope = serde_json::json!({"type":"Audio","future_envelope":{"keep":true},
        "options":{"sample_rate":22050,"normalize":false,"looping":false}});
    let s: crate::import_settings::Settings = serde_json::from_value(envelope).unwrap();
    let written = serde_json::to_value(&s).unwrap();
    assert_eq!(written["future_envelope"]["keep"], true);
    assert_eq!(
        serde_json::from_value::<crate::import_settings::Settings>(written).unwrap(),
        s
    );
}

#[test]
fn audio_role_does_not_choose_residency_or_change_legacy_samples() {
    use audio_import::{AudioRole, LoadMode, Quality, Settings};
    let source = audio_import::test_wav();
    let baseline = audio_import::convert(&source, &Settings::default())
        .unwrap()
        .adpcm;
    for role in [
        AudioRole::Sfx,
        AudioRole::Music,
        AudioRole::Ambience,
        AudioRole::Dialogue,
    ] {
        let mut s = Settings {
            role,
            load_mode: LoadMode::Resident,
            ..Default::default()
        };
        assert_eq!(audio_import::convert(&source, &s).unwrap().adpcm, baseline);
        s.load_mode = LoadMode::Stream;
        s.quality = Quality::Medium;
        assert!(s.is_streamed());
        assert_eq!(s.rate(), 37800);
        s.validate_psx().unwrap();
        s.load_mode = LoadMode::Auto;
        assert_eq!(s.is_streamed(), role != AudioRole::Sfx);
    }
    let s = Settings {
        channels: 2,
        ..Default::default()
    };
    s.validate().unwrap(); // Valid portable authoring, unsupported PSX resident stereo.
    assert!(s.target_result().error.unwrap().contains("mono"));
    assert_eq!(s.load_mode, LoadMode::Resident);
    assert!(audio_import::convert(&source, &s).is_err());
}

#[test]
fn audio_package_reimport_and_duplicate_preserve_unknown_metadata() {
    let root = crate::workspace::tests::temp("audio-migration-roundtrip");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(root.join("assets/tone.wav"), audio_import::test_wav()).unwrap();
    let mut c = assets::prepare(
        &root,
        "assets/tone.wav",
        "assets/tone.epokasset",
        Default::default(),
        None,
        false,
    )
    .unwrap();
    c.package.meta.extra.insert(
        "future_metadata".into(),
        serde_json::json!({"opaque":"keep"}),
    );
    let id = assets::commit(c).unwrap();
    let index = assets::scan(&root, &mut Default::default());
    let old = index.resolve(id).unwrap();
    let c = assets::prepare(
        &root,
        "",
        "assets/tone.epokasset",
        old.meta.settings.audio().unwrap().clone(),
        Some(old),
        true,
    )
    .unwrap();
    assert_eq!(assets::commit(c).unwrap(), id);
    let p = assets::Package::load(&old.path).unwrap();
    assert_eq!(p.meta.extra["future_metadata"]["opaque"], "keep");
    let copy = root.join("assets/copy.epokasset");
    assert_ne!(assets::duplicate(old, &copy).unwrap(), id);
    let duplicate = assets::Package::load(&copy).unwrap();
    assert_eq!(duplicate.meta.extra, p.meta.extra);
    assert_eq!(duplicate.source, p.source);
}

#[test]
fn audio_legacy_golden_outputs() {
    let root = crate::workspace::tests::temp("audio-legacy-golden");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let source = audio_import::test_wav();
    // This is the actual v1 untagged package shape, independent of current serializers.
    let id = uuid::Uuid::from_u128(1);
    let meta = serde_json::json!({
        "version":1,"id":id,"kind":"AudioClip","importer_version":1,
        "source":"assets/tone.wav","source_hash":assets::hash(&source),
        "settings":{"sample_rate":22050,"normalize":false,"looping":false}
    });
    let metadata = serde_json::to_vec(&meta).unwrap();
    let mut legacy = b"EPOKAS01".to_vec();
    legacy.extend((metadata.len() as u32).to_le_bytes());
    legacy.extend((source.len() as u32).to_le_bytes());
    legacy.extend(metadata);
    legacy.extend(&source);
    let path = root.join("assets/tone.epokasset");
    std::fs::write(&path, &legacy).unwrap();
    let package = assets::Package::load(&path).unwrap();
    let settings = package.meta.settings.audio().unwrap();
    let mut hashes = std::collections::BTreeMap::new();
    hashes.insert("legacy_package".to_owned(), assets::hash(&legacy));
    for (name, options) in [
        ("resident", settings.clone()),
        (
            "resident_loop",
            audio_import::Settings {
                looping: true,
                ..settings.clone()
            },
        ),
        (
            "resident_trim_normalize",
            audio_import::Settings {
                trim_start: 0.02,
                trim_end: Some(0.08),
                normalize: true,
                sample_rate: 11025,
                ..settings.clone()
            },
        ),
    ] {
        hashes.insert(
            name.into(),
            assets::hash(&audio_import::convert(&source, &options).unwrap().adpcm),
        );
    }
    let mut scene = crate::scene::Scene::default();
    let mut entity = crate::scene::Actor::cube("Tone".into());
    entity.audio = Some(crate::audio::AudioSource {
        clip: Some(id),
        ..Default::default()
    });
    scene.actors.push(entity);
    let index = assets::scan(&root, &mut Default::default());
    let build = root.join("build");
    std::fs::create_dir_all(&build).unwrap();
    crate::audio::stage(&root, &scene, &build, &index).unwrap();
    hashes.insert(
        "resident_bank_header".into(),
        assets::hash(&std::fs::read(build.join("audio-bank.hh")).unwrap()),
    );
    for (rate, channels) in [(37800, 2), (37800, 1), (18900, 2), (18900, 1)] {
        let options: audio_import::Settings = serde_json::from_value(serde_json::json!({
            "usage":"Music","sample_rate":rate,"channels":channels,"normalize":false,"looping":true
        }))
        .unwrap();
        let mut raw = vec![0; crate::music::SECTOR * 2];
        for sector in raw.chunks_exact_mut(crate::music::SECTOR) {
            sector[..8].copy_from_slice(&[0, 0, 0x64, 0, 0, 0, 0x64, 0]);
        }
        hashes.insert(
            format!("xa_layout_{rate}_{channels}"),
            assets::hash(&crate::music::interleave(&raw, &options).unwrap()),
        );
    }
    let expected = serde_json::from_str::<std::collections::BTreeMap<String, String>>(
        include_str!("../tests/fixtures/audio-legacy-golden.json"),
    )
    .unwrap();
    assert_eq!(hashes, expected);
}
