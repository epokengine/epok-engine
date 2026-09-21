#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]
// Debug desktop launches are GUI-only too; the test harness keeps its console.
// Redirected CLI output is still available to build tools and diagnostics.
mod actor_components;
mod actor_document;
mod actor_scripts;
mod actor_workflow;
mod artifact_dependencies;
mod artifact_dependency_ui;
mod asset_inspector;
mod asset_manager;
mod asset_ui;
mod assets;
mod audio;
#[cfg(test)]
mod audio_contract_tests;
mod audio_decode;
mod audio_import;
mod audio_ir;
mod bank_compat;
mod bitmap_font;
mod blueprint;
mod blueprint_asset;
mod blueprint_compile;
mod blueprint_debug;
mod blueprint_debug_ui;
mod blueprint_dependencies;
mod blueprint_editor;
mod blueprint_ir;
mod blueprint_playback;
mod blueprint_refs;
mod blueprint_spawn;
mod blueprint_templates;
mod blueprint_workflow;
mod branding;
mod bridge;
mod brush;
mod build_inputs;
mod build_report;
mod busy_ui;
mod collision;
mod collision_editor;
mod console;
mod content_preview;
mod controls;
mod controls_ui;
mod dependencies;
mod disc;
mod document;
mod editor;
mod effects;
mod export;
mod export_ui;
mod file_watch;
mod gizmo;
mod gui;
mod hub;
mod hud;
mod hud_editor;
mod hud_native;
mod hud_simulation;
mod import_settings;
mod inspector_theme;
mod instrument_dsp;
mod instrument_ir;
mod instrument_modulation;
mod instrument_preview;
mod instrument_samples;
mod instrument_selection;
mod instrument_source_preview;
mod instrument_voice;
mod library_preview;
mod lighting;
mod lighting_editor;
mod loading;
mod lua_aot;
mod lua_api_stub;
mod lua_asset;
mod lua_bytecode;
mod lua_compile;
mod lua_dependencies;
mod lua_frontend;
mod lua_identity;
mod lua_vm;
mod mcp;
mod mcp_stdio;
#[cfg(test)]
mod mcp_tests;
mod mcp_tools;
mod memory;
mod memory_ui;
mod mesh;
mod mesh_compile;
mod mesh_editor;
mod mesh_ops;
mod midi;
mod model_import;
mod music;
mod music_conversion_ui;
mod native;
mod native_metadata;
mod native_music;
mod native_play;
mod navigation;
mod navigation_geometry;
mod obj_import;
mod object_model;
mod operation;
mod palette;
mod particle_effect;
mod particle_effect_editor;
mod particle_effect_preview;
mod particle_effect_scene;
mod particles;
mod picking;
mod pipeline;
mod platform;
mod play;
mod play_cache;
mod play_ui;
mod playback_staging;
mod preview_audio;
mod project;
mod project_browser;
mod psx_library;
mod psx_library_asset;
#[cfg(test)]
mod psx_library_asset_tests;
mod psx_library_wire;
mod psx_loop_quality;
mod psx_music_optimizer;
mod psx_music_settings;
mod psx_sequence;
mod reflection;
mod reflection_schema;
#[cfg(test)]
mod runtime_api_tests;
mod scene;
mod scene_bank;
mod scene_dependencies;
mod scene_gpu;
mod scene_loading;
mod scene_view_mode;
mod script_backend;
mod script_ir;
mod script_values;
mod scripts;
mod sequence;
mod sequence_compat;
mod sequence_export;
mod sequence_ir;
mod sequence_preview;
mod sequence_stream;
mod sequencer;
mod serial;
mod serial_support;
mod serial_terminal;
mod serial_ui;
mod settings;
mod settings_ui;
mod sf2;
mod shadows;
mod skeletal;
mod skeletal_compile;
#[cfg(test)]
mod skeletal_tests;
mod skeletal_ui;
mod sound_bank;
mod soundfont_asset;
mod sprites;
mod sprites_editor;
mod spu_encoder;
mod staging_files;
mod streaming;
mod terrain;
mod terrain_compile;
mod terrain_editor;
mod texture;
mod third_person;
mod timeline;
mod timeline_adapters;
mod timeline_compile;
mod timeline_curve;
mod timeline_editor;
mod timeline_ir;
mod timeline_runtime;
mod timeline_scene;
mod timeline_scene_preview;
mod timeline_section;
mod transform;
mod vab_import;
mod viewport;
mod workspace;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn parse_animation_storage(value: &str) -> Result<skeletal::AnimationStorage, String> {
    match value {
        "rigid-gte" | "RigidGte" => Ok(skeletal::AnimationStorage::RigidGte),
        "baked-vertices" | "BakedVertices" => Ok(skeletal::AnimationStorage::BakedVertices),
        _ => Err("Animation storage must be rigid-gte or baked-vertices".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|a| a == "--mcp-stdio") {
        return mcp_stdio::run();
    }
    // Installer/diagnostic entry points do not require a project or contact the console.
    if args
        .iter()
        .any(|a| a == "--prepare-serial-tools" || a == "--list-serial-ports")
    {
        let action = if args.iter().any(|a| a == "--prepare-serial-tools") {
            serial_ui::Action::Prepare
        } else {
            serial_ui::Action::Scan
        };
        let job = pipeline::Job::serial_setup(play::Serial::default(), action);
        loop {
            match job.events.recv()? {
                pipeline::Event::Log(s) | pipeline::Event::SerialStatus(s) => println!("{s}"),
                pipeline::Event::SerialPorts(ports) => {
                    println!("{}", serde_json::to_string_pretty(&ports)?)
                }
                pipeline::Event::Finished(result) => return result.map_err(Into::into),
                _ => {}
            }
        }
    }
    let requested_root = args
        .windows(2)
        .find(|v| v[0] == "--project")
        .map(|v| PathBuf::from(&v[1]))
        .or_else(|| {
            args.iter()
                .skip(1)
                .find(|argument| {
                    std::path::Path::new(argument)
                        .extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case("epokproject"))
                })
                .map(PathBuf::from)
        });
    let screenshot = args
        .windows(2)
        .find(|v| v[0] == "--screenshot")
        .map(|v| PathBuf::from(&v[1]));
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--create-project") {
        let destination = std::path::absolute(&pair[1])?;
        let name = args
            .windows(2)
            .find(|v| v[0] == "--name")
            .map(|v| v[1].as_str())
            .or_else(|| destination.file_name().and_then(|v| v.to_str()))
            .ok_or("Project name required")?;
        let template = match args
            .windows(2)
            .find(|v| v[0] == "--template")
            .map(|v| v[1].as_str())
            .unwrap_or("basic")
        {
            "basic" => workspace::Template::Basic,
            "sample" => workspace::Template::Sample,
            "third-person" => workspace::Template::ThirdPerson,
            other => {
                return Err(format!(
                    "Unknown template: {other}. Use basic, sample or third-person."
                )
                .into());
            }
        };
        let project = workspace::create(&destination, name, template)?;
        println!("Created project: {}", project.root.display());
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--migrate-project") {
        let descriptor = workspace::migrate_legacy(std::path::Path::new(&pair[1]))?;
        println!("Migrated project descriptor: {}", descriptor.display());
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--recover-project") {
        let prefer = args
            .windows(2)
            .find(|v| v[0] == "--prefer")
            .map(|v| v[1].as_str())
            .ok_or("Recovery requires --prefer descriptor or --prefer legacy.")?;
        let active = workspace::recover_migration(std::path::Path::new(&pair[1]), prefer)?;
        println!("Recovered active project settings: {}", active.display());
        return Ok(());
    }
    let headless = args.iter().any(|a| {
        [
            "--build-psx",
            "--play-psx",
            "--play-native",
            "--analyze-memory",
            "--generate-asset-report",
            "--bake-lighting",
            "--bake-navigation",
            "--add-actor",
            "--profile-scene",
            "--profile-scene-cpu",
            "--profile-editor",
            "--import-audio",
            "--create-sound-bank",
            "--create-starter-bank",
            "--import-sound-bank",
            "--default-sound-bank",
            "--import-fbx",
            "--import-obj",
            "--import-texture",
            "--reimport-asset",
            "--duplicate-music-sequence",
            "--render-music-sequence",
            "--scan-assets",
            "--inspect-audio-source",
            "--reflect",
            "--new-script",
            "--new-blueprint",
            "--new-lua-class",
            "--compile-blueprints",
            "--new-timeline",
            "--install-timeline-adapters",
            "--compile-timelines",
            "--new-particle-effect",
            "--validate-particle-effects",
            "--preview-particle-effect",
            "--preview-hud",
            "--export-psx",
        ]
        .contains(&a.as_str())
    });
    let Some(requested_root) = requested_root else {
        if headless {
            return Err("Select a project with --project <folder-or-descriptor>.".into());
        }
        return platform::run(None, screenshot, false, None);
    };
    if !headless {
        return platform::run(
            Some(loading::Request::Open(requested_root)),
            screenshot,
            false,
            None,
        );
    }
    let project = workspace::Project::open(&requested_root)?;
    let root = project.root.clone();
    if args.iter().any(|a| a == "--install-timeline-adapters") {
        println!("{}", timeline_adapters::install(&root)?.display());
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--preview-hud") {
        let frames = args
            .windows(2)
            .find(|v| v[0] == "--frames")
            .map(|v| v[1].parse::<u32>())
            .transpose()?
            .unwrap_or(120);
        let output = args
            .windows(2)
            .find(|v| v[0] == "--output")
            .map(|v| PathBuf::from(&v[1]))
            .ok_or("--preview-hud requires --output image.png")?;
        println!(
            "{}",
            serde_json::to_string(&hud_simulation::capture(
                &root,
                &root.join(&pair[1]),
                frames,
                &output
            )?)?
        );
        return Ok(());
    }
    if let Some(pair) = args
        .windows(2)
        .find(|v| v[0] == "--preview-particle-effect")
    {
        let steps = args
            .windows(2)
            .find(|v| v[0] == "--steps")
            .map(|v| v[1].parse::<u32>())
            .transpose()?
            .unwrap_or(482);
        println!(
            "{}",
            serde_json::to_string(&particle_effect_preview::trace(
                &root,
                &root.join(&pair[1]),
                steps,
            )?)?
        );
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--new-particle-effect") {
        let preset = args
            .windows(2)
            .find(|p| p[0] == "--preset")
            .map(|p| particle_effect_editor::Preset::parse(&p[1]))
            .transpose()?;
        println!(
            "{}",
            particle_effect::create_from_preset(&root, &pair[1], preset)?.display()
        );
        return Ok(());
    }
    if args.iter().any(|v| v == "--validate-particle-effects") {
        println!(
            "{}",
            serde_json::to_string_pretty(&particle_effect::validate_project(&root)?)?
        );
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--new-timeline") {
        println!("{}", timeline::create(&root, &pair[1])?.display());
        return Ok(());
    }
    if args.iter().any(|v| v == "--compile-timelines") {
        let results = timeline_compile::compile_current_project(&root)?;
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--new-blueprint") {
        let native = scripts::native_catalog(&root)?;
        let mut registry = blueprint::native_registry(&root, &native)?;
        if let Some(compiled) = scripts::compile_blueprints(&root, &native)? {
            registry = compiled.registry;
        }
        let parent = args
            .windows(2)
            .find(|v| v[0] == "--parent")
            .map(|v| v[1].as_str())
            .unwrap_or("epok::ActorComponent");
        let class = registry
            .classes
            .get(parent)
            .or_else(|| registry.named(parent))
            .ok_or("Unknown Blueprint parent; use a reflected class ID or qualified name.")?;
        let folder = args
            .windows(2)
            .find(|v| v[0] == "--folder")
            .map(|v| v[1].as_str())
            .unwrap_or("");
        let path =
            blueprint_workflow::create(&root, &registry, &pair[1], folder, &class.id, false)?;
        println!("{}", path.display());
        return Ok(());
    }
    if args.iter().any(|v| v == "--compile-blueprints") {
        let native = scripts::native_catalog(&root)?;
        if let Some(compiled) = scripts::compile_blueprints(&root, &native)? {
            compiled
                .artifacts
                .stage(&root, &root.join(".epok/blueprints"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"schema_version":1,"classes":compiled.registry.classes.values().collect::<Vec<_>>(),"sources":compiled.artifacts.files.keys().collect::<Vec<_>>()})
                )?
            );
        } else {
            println!("{{\"schema_version\":1,\"classes\":[],\"sources\":[]}}");
        }
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--new-script") {
        let parent = args
            .windows(2)
            .find(|v| v[0] == "--parent")
            .map(|v| v[1].as_str())
            .unwrap_or("ActorComponent");
        if let Some(folder) = args.windows(2).find(|v| v[0] == "--folder") {
            scripts::create_in(&root, &pair[1], &folder[1], parent, false)?;
        } else {
            scripts::create_named(&root, &pair[1], parent)?;
        }
        println!("Created {} : {parent}", pair[1]);
        return Ok(());
    }
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--new-lua-class") {
        let parent = args
            .windows(2)
            .find(|v| v[0] == "--parent")
            .map(|v| v[1].as_str())
            .unwrap_or("epok::ActorComponent");
        let folder = args
            .windows(2)
            .find(|v| v[0] == "--folder")
            .map(|v| v[1].as_str())
            .unwrap_or("");
        let path = lua_asset::create_in(&root, &pair[1], folder, parent)?;
        println!("{}", path.display());
        return Ok(());
    }
    if args.iter().any(|a| a == "--export-psx") {
        let scene = scene_dependencies::Input::load(&workspace::startup_scene(&root)?)?;
        println!("{}", export::export_project(&root, &scene)?.display());
        return Ok(());
    }
    if args.iter().any(|a| a == "--reflect") {
        println!(
            "{}",
            serde_json::to_string_pretty(&reflection::discover(&root)?)?
        );
        return Ok(());
    }
    let value = |flag: &str| {
        args.windows(2)
            .find(|v| v[0] == flag)
            .map(|v| v[1].as_str())
    };
    if let Some(sequence) = value("--render-music-sequence") {
        let output =
            value("--output").ok_or("--render-music-sequence requires --output assets/path.wav")?;
        let rendered = sequence_export::render_source_wav(
            &root,
            &assets::inside(&root, sequence)?,
            &assets::inside(&root, output)?,
        )?;
        println!(
            "Rendered MusicSequence source WAV: {output} ({} frames, {} Hz, {} channels)",
            rendered.frames, rendered.rate, rendered.channels
        );
        return Ok(());
    }
    if let Some(sequence) = value("--duplicate-music-sequence") {
        let source_path = assets::inside(&root, sequence)?;
        let destination = value("--asset")
            .ok_or("--duplicate-music-sequence requires --asset assets/path.epokasset")?;
        let destination_path = assets::inside(&root, destination)?;
        if source_path == destination_path {
            return Err("The compact MusicSequence needs a new asset destination".into());
        }
        let index = assets::scan(&root, &mut Default::default());
        let source_package = assets::Package::load(&source_path)?;
        if source_package.meta.kind != assets::Kind::MusicSequence {
            return Err("--duplicate-music-sequence requires a MusicSequence asset".into());
        }
        let source_record = index.resolve(source_package.meta.id)?;
        let mut settings = source_package.meta.settings.sequence()?.clone();
        let mut recipe = psx_music_settings::Recipe::from_settings(&settings)?;
        recipe.preset = psx_music_settings::Preset::Custom;
        recipe.bank_budget_bytes = value("--bank-budget")
            .ok_or("--duplicate-music-sequence requires --bank-budget <SPU bytes>")?
            .parse()?;
        recipe.optimization.minimum_sample_rate = value("--minimum-sample-rate")
            .map(str::parse)
            .transpose()?
            .unwrap_or(400);
        recipe.optimization.allow_lower_rate = true;
        recipe.optimization.max_candidates = 32;
        recipe.store(&mut settings)?;
        let candidate = sequence::prepare(
            &root,
            &source_record.meta.source,
            destination,
            settings,
            None,
            false,
        )?;
        let fitted =
            psx_music_settings::Recipe::from_settings(candidate.package.meta.settings.sequence()?)?;
        let id = assets::commit(candidate)?;
        println!(
            "Duplicated compact MusicSequence {id}: {} byte PSX bank budget, fitted maximum {} Hz",
            fitted.bank_budget_bytes, fitted.max_sample_rate
        );
        return Ok(());
    }
    if let Some(source) = value("--inspect-audio-source") {
        let explicit_profile = value("--sequence-profile")
            .map(sequence::SourceProfile::from_id)
            .transpose()?;
        let bytes = assets::read_bounded(&assets::inside(&root, source)?)?;
        if bytes.starts_with(b"EPOKAS01") {
            let package = assets::Package::load(&assets::inside(&root, source)?)?;
            if package.meta.kind == assets::Kind::SoundBank {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&bank_compat::inspect(&package)?)?
                );
            } else {
                let profile = explicit_profile.or(package
                    .meta
                    .settings
                    .sequence()?
                    .source_selection
                    .as_ref()
                    .map(|s| s.profile));
                println!(
                    "{}",
                    serde_json::to_string_pretty(&sequence::catalog_source(
                        &package.source,
                        profile
                    )?)?
                );
            }
        } else if bank_compat::has_header(&bytes) || value("--vb").is_some() {
            let companion = value("--vb")
                .map(|name| assets::inside(&root, name).and_then(|p| assets::read_bounded(&p)))
                .transpose()?;
            let bank = vab_import::parse(
                vab_import::Input {
                    bytes: &bytes,
                    label: source,
                    rights: value("--provenance").unwrap_or(""),
                },
                companion.as_ref().map(|bytes| vab_import::Input {
                    bytes,
                    label: value("--vb").unwrap(),
                    rights: value("--provenance").unwrap_or(""),
                }),
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"profile":bank.profile,"header":bank.header,"programs":bank.programs,"samples":bank.samples.iter().map(|s|serde_json::json!({"id":s.id,"encoded":s.encoded,"sha256":s.encoded_sha256,"original_hz":s.original_sample_rate_hz,"frames":s.decoded.pcm.len(),"termination":s.decoded.termination,"loop":s.decoded.loop_region})).collect::<Vec<_>>(),"compatibility":vab_import::assess_current_sound_bank(&bank)})
                )?
            );
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&sequence::catalog_source(&bytes, explicit_profile)?)?
            );
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--scan-assets") {
        let index = assets::scan(&root, &mut Default::default());
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "assets":index.assets.values().flatten().map(|r|serde_json::json!({"path":assets::path_string(&root,&r.path),"id":r.meta.id,"source":r.meta.source,"usable":index.resolve(r.meta.id).is_ok()})).collect::<Vec<_>>(),
                "sources":index.sources.keys().collect::<Vec<_>>(),"problems":index.problems
            }))?
        );
        return Ok(());
    }
    if let Some(source) = value("--import-obj") {
        let destination =
            value("--asset").ok_or("OBJ import requires --asset assets/path.epokasset")?;
        let scale = value("--scale")
            .map(str::parse::<f32>)
            .transpose()?
            .unwrap_or(1.);
        println!(
            "Imported static OBJ: {}",
            obj_import::import(&root, source, destination, scale)?
        );
        return Ok(());
    }
    if let Some(source) = value("--import-fbx") {
        let destination = value("--asset")
            .map(str::to_owned)
            .unwrap_or_else(|| model_import::destination(source));
        let storage = value("--animation-storage")
            .map(parse_animation_storage)
            .transpose()?;
        let candidate = model_import::prepare(&root, source, &destination, None, false, storage)?;
        println!("Imported FBX: {}", model_import::commit(candidate)?);
        return Ok(());
    }
    if let Some(source) = value("--import-texture") {
        let destination = value("--asset").map(str::to_owned).unwrap_or_else(|| {
            std::path::Path::new(source)
                .with_extension("epokasset")
                .to_string_lossy()
                .into_owned()
        });
        println!(
            "Imported Texture: {}",
            assets::commit(texture::prepare(&root, source, &destination, None, false)?)?
        );
        return Ok(());
    }
    if let Some(path) = value("--reimport-asset") {
        let path = assets::inside(&root, path)?;
        let p = assets::Package::load(&path)?;
        if p.meta.kind == assets::Kind::Texture {
            let index = assets::scan(&root, &mut Default::default());
            let r = index.resolve(p.meta.id)?;
            let source = index
                .linked_source(r)?
                .map_or(p.meta.source.as_str(), |s| s.path.as_str());
            println!(
                "Reimported Texture: {}",
                assets::commit(texture::prepare(
                    &root,
                    source,
                    &assets::path_string(&root, &path),
                    Some(r),
                    args.iter().any(|a| a == "--snapshot")
                )?)?
            );
            return Ok(());
        }
        if p.meta.kind == assets::Kind::ModelSource {
            let index = assets::scan(&root, &mut Default::default());
            let r = index.resolve(p.meta.id)?;
            let source = index
                .linked_source(r)?
                .map_or(p.meta.source.as_str(), |s| s.path.as_str());
            let candidate = model_import::prepare(
                &root,
                source,
                &assets::path_string(&root, &path),
                Some(r),
                args.iter().any(|a| a == "--snapshot"),
                value("--animation-storage")
                    .map(parse_animation_storage)
                    .transpose()?,
            )?;
            println!("Reimported FBX: {}", model_import::commit(candidate)?);
            return Ok(());
        }
    }
    if let Some(destination) = value("--create-starter-bank") {
        let id = sound_bank::publish_starter(sound_bank::starter_candidates(&root, destination)?)?;
        println!(
            "Created Retro Starter SoundBank {id}. Program 0, original generated triangle (MIT). Assign it explicitly to a sequence or Project Default SoundBank."
        );
        return Ok(());
    }
    if let Some(destination) = value("--create-sound-bank") {
        let sample = value("--sample")
            .ok_or("Use --sample <AudioClip UUID>")?
            .parse::<uuid::Uuid>()?;
        let index = assets::scan(&root, &mut Default::default());
        if index.resolve(sample)?.meta.kind != assets::Kind::AudioClip {
            return Err("SoundBank samples must be AudioClips".into());
        }
        let mut zone = sound_bank::Zone::new(sample);
        if let Some(root_key) = value("--root-key") {
            zone.root_key = root_key.parse()?;
        }
        if let Some(start) = value("--sample-loop-start") {
            zone.sample_loop = Some([
                start.parse()?,
                value("--sample-loop-end")
                    .ok_or("Supply --sample-loop-end in original source frames")?
                    .parse()?,
            ]);
        }
        let settings = sound_bank::Settings {
            programs: vec![sound_bank::Program {
                program: value("--program").unwrap_or("0").parse()?,
                drum_key: value("--drum-key").map(str::parse).transpose()?,
                zones: vec![zone],
                extra: Default::default(),
            }],
            provenance: value("--provenance").unwrap_or("").into(),
            ..Default::default()
        };
        println!(
            "Created SoundBank: {}",
            sound_bank::create(&root, destination, settings)?
        );
        return Ok(());
    }
    if let Some(source) = value("--import-sound-bank") {
        let bytes = assets::read_bounded(&assets::inside(&root, source)?)?;
        let destination = value("--asset").map(str::to_owned).unwrap_or_else(|| {
            PathBuf::from(source)
                .with_extension("epokasset")
                .to_string_lossy()
                .into()
        });
        if sf2::has_header(&bytes) {
            let settings = sound_bank::Settings {
                provenance: value("--provenance").unwrap_or("").into(),
                ..Default::default()
            };
            let candidate =
                soundfont_asset::prepare(&root, source, &destination, settings, None, false)?;
            println!(
                "Imported instrument library SoundBank: {}",
                assets::commit(candidate)?
            );
            return Ok(());
        }
        if bank_compat::has_header(&bytes)
            || value("--vb").is_some()
            || bank_compat::source_candidate(Path::new(source))
        {
            let settings = sound_bank::Settings {
                provenance: value("--provenance").unwrap_or("").into(),
                ..Default::default()
            };
            let candidate = bank_compat::prepare(
                &root,
                source,
                value("--vb"),
                &destination,
                settings,
                None,
                false,
            )?;
            let id = bank_compat::commit(candidate)?;
            println!(
                "Imported Sony SoundBank {id}. {}",
                bank_compat::PLAYBACK_BLOCKER
            );
            return Ok(());
        }
        let settings: sound_bank::Settings = document::from_slice(&bytes)?;
        let candidate = assets::prepare_portable(
            &root,
            source,
            &destination,
            import_settings::Settings::SoundBank(settings),
            None,
            false,
        )?;
        println!("Imported SoundBank: {}", assets::commit(candidate)?);
        return Ok(());
    }
    if let Some(bank) = value("--default-sound-bank") {
        let id = if bank == "none" {
            None
        } else {
            Some(bank.parse::<uuid::Uuid>()?)
        };
        if let Some(id) = id {
            let index = assets::scan(&root, &mut Default::default());
            if index.resolve(id)?.meta.kind != assets::Kind::SoundBank {
                return Err("Project default must be a SoundBank".into());
            }
        }
        let mut manifest = workspace::read_manifest(&root)?;
        manifest.default_sound_bank = id;
        workspace::save_manifest(&root, &manifest)?;
        println!("Project Default SoundBank: {bank}");
        return Ok(());
    }
    if value("--import-audio").is_some() || value("--reimport-asset").is_some() {
        let index = assets::scan(&root, &mut Default::default());
        let existing = value("--reimport-asset")
            .map(|path| {
                let path = assets::inside(&root, path)?;
                let package = assets::Package::load(&path)?;
                index.resolve(package.meta.id).cloned()
            })
            .transpose()?;
        let source = value("--import-audio")
            .map(str::to_owned)
            .or_else(|| {
                existing.as_ref().and_then(|r| {
                    index
                        .linked_source(r)
                        .ok()
                        .flatten()
                        .map(|s| s.path.clone())
                })
            })
            .unwrap_or_default();
        let destination = value("--asset")
            .map(str::to_owned)
            .or_else(|| {
                existing
                    .as_ref()
                    .map(|r| assets::path_string(&root, &r.path))
            })
            .unwrap_or_else(|| {
                PathBuf::from(&source)
                    .with_extension("epokasset")
                    .to_string_lossy()
                    .into_owned()
            });
        let source_bytes = if args.iter().any(|a| a == "--snapshot") {
            existing
                .as_ref()
                .map(|r| assets::Package::load(&r.path).map(|p| p.source))
                .transpose()?
        } else if source.is_empty() {
            None
        } else {
            Some(assets::read_bounded(&assets::inside(&root, &source)?)?)
        };
        if source_bytes
            .as_ref()
            .is_some_and(|bytes| sf2::has_header(bytes))
            || existing.as_ref().is_some_and(|record| {
                record
                    .meta
                    .settings
                    .sound_bank()
                    .is_ok_and(|settings| settings.library.is_some())
            })
        {
            let mut settings = existing
                .as_ref()
                .map(|record| record.meta.settings.sound_bank().cloned())
                .transpose()?
                .unwrap_or_default();
            if let Some(provenance) = value("--provenance") {
                settings.provenance = provenance.into();
            }
            let candidate = soundfont_asset::prepare(
                &root,
                &source,
                &destination,
                settings,
                existing.as_ref(),
                args.iter().any(|a| a == "--snapshot"),
            )?;
            println!(
                "Imported instrument library SoundBank: {}",
                assets::commit(candidate)?
            );
            return Ok(());
        }
        if source_bytes
            .as_ref()
            .is_some_and(|b| bank_compat::has_header(b))
            || existing
                .as_ref()
                .is_some_and(|r| bank_compat::parts(r).is_some())
        {
            let mut settings = existing
                .as_ref()
                .map(|r| r.meta.settings.sound_bank().cloned())
                .transpose()?
                .unwrap_or_default();
            if let Some(provenance) = value("--provenance") {
                settings.provenance = provenance.into();
            }
            let candidate = bank_compat::prepare(
                &root,
                &source,
                value("--vb"),
                &destination,
                settings,
                existing.as_ref(),
                args.iter().any(|a| a == "--snapshot"),
            )?;
            println!(
                "Imported Sony SoundBank {}. {}",
                bank_compat::commit(candidate)?,
                bank_compat::PLAYBACK_BLOCKER
            );
            return Ok(());
        }
        let midi = existing
            .as_ref()
            .is_some_and(|r| r.meta.kind == assets::Kind::MusicSequence)
            || source_bytes.as_ref().is_some_and(|b| {
                b.starts_with(b"MThd")
                    || b.starts_with(b"pQES")
                    || sequence::catalog_source(b, None).is_ok()
            })
            || value("--sequence-profile").is_some()
            || std::path::Path::new(&source).extension().is_some_and(|e| {
                matches!(
                    e.to_ascii_lowercase().to_str(),
                    Some("mid" | "midi" | "seq" | "sep")
                )
            });
        if midi {
            let mut settings = existing
                .as_ref()
                .map(|r| r.meta.settings.sequence().cloned())
                .transpose()?
                .unwrap_or_default();
            if let Some(bytes) = &source_bytes {
                let profile = value("--sequence-profile")
                    .map(sequence::SourceProfile::from_id)
                    .transpose()?;
                let catalog = sequence::catalog_source(bytes, profile)?;
                if let Some(profile) = catalog.profile
                    && (settings.source_selection.is_none()
                        || value("--song-id").is_some()
                        || value("--song-index").is_some()
                        || value("--sequence-profile").is_some())
                {
                    settings.source_selection = Some(sequence::select_source(
                        bytes,
                        profile,
                        value("--song-id").map(str::parse).transpose()?,
                        value("--song-index").map(str::parse).transpose()?,
                    )?);
                }
            }
            if [
                "--rate",
                "--channels",
                "--trim-start",
                "--trim-end",
                "--normalize",
                "--audio-usage",
            ]
            .iter()
            .any(|flag| args.iter().any(|a| a == flag))
            {
                return Err("Sample rate, channels, trim, normalization and legacy usage apply to sampled AudioClips. For MIDI use --sound-bank, --voice-limit and --sequence-loop.".into());
            }
            if let Some(role) = value("--audio-role") {
                settings.role = match role {
                    "sfx" => audio_import::AudioRole::Sfx,
                    "music" => audio_import::AudioRole::Music,
                    "ambience" => audio_import::AudioRole::Ambience,
                    "dialogue" => audio_import::AudioRole::Dialogue,
                    _ => return Err("Use --audio-role sfx|music|ambience|dialogue".into()),
                };
            }
            if let Some(mode) = value("--load-mode") {
                settings.load_mode = match mode {
                    "auto" => audio_import::LoadMode::Auto,
                    "resident" => audio_import::LoadMode::Resident,
                    "stream" => audio_import::LoadMode::Stream,
                    _ => return Err("Use --load-mode auto|resident|stream".into()),
                };
            }
            if let Some(bank) = value("--sound-bank") {
                settings.sound_bank = if bank == "default" {
                    None
                } else {
                    Some(bank.parse()?)
                };
            }
            if let Some(profile) = value("--midi-interpretation") {
                if settings.source_selection.is_some() {
                    return Err("--midi-interpretation applies only to Standard MIDI Files".into());
                }
                settings.midi_profile = match profile {
                    "legacy-v1" => midi::MidiProfile::LegacyV1,
                    "musical-v2" => midi::MidiProfile::MusicalV2,
                    _ => return Err("Use --midi-interpretation legacy-v1|musical-v2".into()),
                };
            }
            if let Some(limit) = value("--voice-limit") {
                settings.voice_limit = if limit == "auto" {
                    None
                } else {
                    Some(limit.parse()?)
                };
            }
            if let Some(path) = value("--psx-music-recipe") {
                let recipe: psx_music_settings::Recipe =
                    document::from_slice(&assets::read_bounded(&assets::inside(&root, path)?)?)?;
                recipe.store(&mut settings)?;
            }
            if args.iter().any(|a| a == "--loop") {
                settings.loop_mode = sequence::LoopMode::Whole;
            }
            if let Some(mode) = value("--sequence-loop") {
                settings.loop_mode = match mode {
                    "off" => sequence::LoopMode::Off,
                    "whole" => sequence::LoopMode::Whole,
                    "markers" => sequence::LoopMode::Markers,
                    _ => return Err("Use --sequence-loop off|whole|markers".into()),
                };
            }
            if args.iter().any(|a| a == "--ignore-unsupported") {
                settings.ignore_unsupported = true;
            }
            let candidate = sequence::prepare(
                &root,
                &source,
                &destination,
                settings,
                existing.as_ref(),
                args.iter().any(|a| a == "--snapshot"),
            )?;
            let report = sequence::decode_source(
                &candidate.package.source,
                candidate.package.meta.settings.sequence()?,
            )?;
            let id = assets::commit(candidate)?;
            println!(
                "Imported MusicSequence {id}: {} events, {:.3} s, peak {} logical voices",
                report.events.len(),
                report.duration_micros as f64 / 1_000_000.,
                report.peak_polyphony
            );
            for diagnostic in report.diagnostics {
                println!(
                    "Track {}, tick {}: {}",
                    diagnostic.track + 1,
                    diagnostic.tick,
                    diagnostic.message
                );
            }
            return Ok(());
        }
        if let Some(record) = existing
            .as_ref()
            .filter(|r| r.meta.kind == assets::Kind::SoundBank)
        {
            let snapshot = args.iter().any(|a| a == "--snapshot");
            if record.meta.settings.sound_bank()?.imported.is_some() {
                let mut settings = record.meta.settings.sound_bank()?.clone();
                if let Some(provenance) = value("--provenance") {
                    settings.provenance = provenance.into();
                }
                let candidate = bank_compat::prepare(
                    &root,
                    &source,
                    value("--vb"),
                    &destination,
                    settings,
                    Some(record),
                    snapshot,
                )?;
                println!(
                    "Reimported Sony SoundBank: {}",
                    bank_compat::commit(candidate)?
                );
                return Ok(());
            }
            let settings = if snapshot {
                record.meta.settings.sound_bank()?.clone()
            } else {
                document::from_slice(&assets::read_bounded(&assets::inside(&root, &source)?)?)?
            };
            let candidate = assets::prepare_portable(
                &root,
                &source,
                &destination,
                import_settings::Settings::SoundBank(settings),
                Some(record),
                snapshot,
            )?;
            println!("Reimported SoundBank: {}", assets::commit(candidate)?);
            return Ok(());
        }
        let mut settings = existing
            .as_ref()
            .map(|r| r.meta.settings.audio().cloned())
            .transpose()?
            .unwrap_or_default();
        if let Some(usage) = value("--audio-usage") {
            (settings.role, settings.load_mode) = match usage {
                "sfx" => (
                    audio_import::AudioRole::Sfx,
                    audio_import::LoadMode::Resident,
                ),
                "bgm" => (
                    audio_import::AudioRole::Music,
                    audio_import::LoadMode::Stream,
                ),
                _ => return Err("Use --audio-usage sfx|bgm".into()),
            };
            settings.quality = audio_import::Quality::Custom;
            settings.sample_rate = if settings.is_streamed() { 37800 } else { 22050 };
            settings.channels = if settings.is_streamed() { 2 } else { 1 };
        }
        if let Some(role) = value("--audio-role") {
            settings.role = match role {
                "sfx" => audio_import::AudioRole::Sfx,
                "music" => audio_import::AudioRole::Music,
                "ambience" => audio_import::AudioRole::Ambience,
                "dialogue" => audio_import::AudioRole::Dialogue,
                _ => return Err("Use --audio-role sfx|music|ambience|dialogue".into()),
            };
        }
        if let Some(mode) = value("--load-mode") {
            settings.load_mode = match mode {
                "auto" => audio_import::LoadMode::Auto,
                "resident" => audio_import::LoadMode::Resident,
                "stream" => audio_import::LoadMode::Stream,
                _ => return Err("Use --load-mode auto|resident|stream".into()),
            };
        }
        if let Some(channels) = value("--channels") {
            settings.channels = channels.parse()?;
        }
        if let Some(start) = value("--trim-start") {
            settings.trim_start = start.parse()?;
        }
        if let Some(end) = value("--trim-end") {
            settings.trim_end = Some(end.parse()?);
        }
        if let Some(rate) = value("--rate") {
            settings.sample_rate = rate.parse()?;
            settings.quality = audio_import::Quality::Custom;
        }
        if args.iter().any(|a| a == "--loop") {
            settings.looping = true;
        }
        if args.iter().any(|a| a == "--normalize") {
            settings.normalize = true;
        }
        let candidate = assets::prepare(
            &root,
            &source,
            &destination,
            settings,
            existing.as_ref(),
            args.iter().any(|a| a == "--snapshot"),
        )?;
        let id = assets::commit(candidate)?;
        println!("Imported {destination}: {id}");
        return Ok(());
    }
    // Automation parity with the scene_add_actor MCP tool: same Model::placeable()
    // gate, same root component, same Scene::validate_with_model before saving.
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--add-actor") {
        let (class, name) = pair[1]
            .rsplit_once(':')
            .ok_or("Use --add-actor <class>:<name>")?;
        let path = match args.windows(2).find(|v| v[0] == "--scene") {
            Some(pair) => root.join(&pair[1]),
            None => workspace::startup_scene(&root)?,
        };
        let mut scene = scene::Scene::load(&path)?;
        let catalog = scripts::catalog(&root)?;
        let model = blueprint::registry_from_catalog(&root, &catalog)
            .model()
            .map_err(|diagnostics| {
                diagnostics
                    .iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
        let class = model
            .placeable()
            .find(|c| c.cpp_name == class || c.id == class)
            .ok_or_else(|| {
                format!(
                    "{class} is not a placeable actor class. Placeable classes: {}",
                    model
                        .placeable()
                        .map(|c| c.cpp_name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        let mut actor = actor_document::ActorInstance::new(
            uuid::Uuid::new_v4(),
            actor_document::ClassReference::new(&class.cpp_name, &class.id),
            name,
        );
        if let Some(root_class) = mcp_tools::root_component_class(&model, class) {
            let mut component =
                actor_document::ComponentInstance::new(uuid::Uuid::new_v4(), root_class, "Root");
            component.root = true;
            actor.components.push(component);
        }
        let id = actor.id;
        scene.actors.push(actor);
        scene.validate_with_model(Some(&model))?;
        scene.save(&path)?;
        println!(
            "Added actor {id} ({}) to {}",
            class.cpp_name,
            path.display()
        );
        return Ok(());
    }
    if args.iter().any(|a| a == "--bake-lighting") {
        let path = workspace::startup_scene(&root)?;
        let mut scene = scene::Scene::load(&path)?;
        scene.bake = Some(lighting::bake(&scene)?);
        scene.save(&path)?;
        println!("Baked lighting saved: {}", path.display());
        return Ok(());
    }
    if args.iter().any(|a| a == "--bake-navigation") {
        let path = args
            .windows(2)
            .find(|v| v[0] == "--scene")
            .map(|v| root.join(&v[1]))
            .map_or_else(|| workspace::startup_scene(&root), Ok)?;
        let mut scene = scene::Scene::load(&path)?;
        scene.navigation = Some(navigation::bake(&scene)?);
        scene.save(&path)?;
        println!(
            "Navigation baked: {} nodes, {}",
            scene.navigation.as_ref().unwrap().nodes.len(),
            path.display()
        );
        return Ok(());
    }
    if args.iter().any(|a| a == "--profile-scene") {
        return scene_gpu::profile(project);
    }
    if args.iter().any(|a| a == "--profile-scene-cpu") {
        let scene = scene::Scene::load(&workspace::startup_scene(&root)?)?;
        let mut view = viewport::View::default();
        let mut samples = Vec::new();
        for _ in 0..40 {
            view.yaw += 0.008;
            let start = Instant::now();
            let image = viewport::render(&scene, Some(1), &view, true, false, false);
            assert_eq!((image.width, image.height), (960, 600));
            let raster = start.elapsed().as_secs_f64() * 1000.;
            let start = Instant::now();
            std::hint::black_box(
                image
                    .pixels
                    .chunks_exact(3)
                    .flat_map(|p| [p[0], p[1], p[2], 255])
                    .collect::<Vec<_>>(),
            );
            samples.push([raster, start.elapsed().as_secs_f64() * 1000.]);
        }
        println!(
            "CPU viewport: raster {:.2} ms, RGB->RGBA {:.2} ms (40 orbit frames, 960x600)",
            samples.iter().map(|s| s[0]).sum::<f64>() / 40.,
            samples.iter().map(|s| s[1]).sum::<f64>() / 40.
        );
        return Ok(());
    }
    if args.iter().any(|a| {
        a == "--build-psx"
            || a == "--play-psx"
            || a == "--play-native"
            || a == "--analyze-memory"
            || a == "--generate-asset-report"
    }) {
        let job = if args.iter().any(|a| a == "--generate-asset-report") {
            let summary =
                build_report::Summary::load(&root, args.iter().any(|a| a == "--blueprint-debug"))
                    .ok_or("Build first before generating an asset report.")?;
            pipeline::Job::report(root.clone(), summary)
        } else {
            let analyze = args.iter().any(|a| a == "--analyze-memory");
            let scene_path = args
                .windows(2)
                .find(|v| v[0] == "--scene")
                .map(|v| root.join(&v[1]))
                .map_or_else(|| workspace::startup_scene(&root), Ok)?;
            let native = args.iter().any(|a| a == "--play-native");
            let mut scene = scene_dependencies::Input::load(&scene_path)?;
            if native {
                let mut profile = play::Profile::load(&root).unwrap_or_default();
                profile.runtime = play::Runtime::NativePc;
                profile.content = play::Content::CurrentScene;
                scene = play::saved_input(&root, &scene_path, profile)?;
            } else if analyze || args.iter().any(|a| a == "--use-play-profile") {
                scene = play::saved_input(
                    &root,
                    &workspace::startup_scene(&root)?,
                    play::Profile::load(&root)?,
                )?;
            }
            if analyze {
                pipeline::Job::analyze(root, scene, args.iter().any(|a| a == "--blueprint-debug"))
            } else {
                pipeline::Job::start_with_debug(
                    root,
                    scene,
                    args.iter()
                        .any(|a| a == "--play-psx" || a == "--play-native"),
                    args.iter().any(|a| a == "--blueprint-debug"),
                )
            }
        };
        let stop_after = args
            .windows(2)
            .find(|v| v[0] == "--stop-after")
            .and_then(|v| v[1].parse::<u64>().ok());
        let mut started = None;
        loop {
            if started.is_some_and(|t: Instant| {
                stop_after.is_some_and(|secs| t.elapsed() > Duration::from_secs(secs))
            }) {
                job.control(pipeline::Control::Stop);
                started = None;
            }
            match job.events.recv_timeout(Duration::from_millis(100)) {
                Ok(pipeline::Event::Log(s)) => println!("{s}"),
                Ok(pipeline::Event::Stage(s) | pipeline::Event::SerialStatus(s)) => println!("{s}"),
                Ok(pipeline::Event::Progress(_) | pipeline::Event::BuildSummary(_)) => {}
                Ok(pipeline::Event::MemoryReport(report)) => println!(
                    "Memory report: {} static RAM; {} build payload. Written to memory-report.json in the build directory.",
                    memory_ui::size(report.ram.used),
                    memory_ui::size(report.files.used)
                ),
                Ok(pipeline::Event::SerialIssue(s)) => eprintln!("{s}"),
                Ok(pipeline::Event::SerialPorts(_)) => {}
                Ok(pipeline::Event::SerialConfigured(serial)) => {
                    let mut preferences = settings::Preferences::load()?;
                    preferences.serial = serial;
                    preferences.save()?;
                }
                Ok(pipeline::Event::LightingBaked(_)) => println!("Vertex lighting ready"),
                Ok(pipeline::Event::Built(p)) => println!("Built {}", p.display()),
                Ok(pipeline::Event::Running(_)) => {
                    println!(
                        "{}",
                        if args.iter().any(|a| a == "--play-native") {
                            "Native PC runtime running"
                        } else {
                            "Emulator running"
                        }
                    );
                    started = Some(Instant::now());
                }
                Ok(pipeline::Event::SerialConnected) => {
                    println!("PSX running over serial");
                    started = Some(Instant::now());
                }
                Ok(pipeline::Event::SerialCommandPending(_)) => {}
                Ok(pipeline::Event::Finished(result)) => return result.map_err(Into::into),
                Ok(pipeline::Event::Paused(_)) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
    platform::run(
        Some(loading::Request::Created(project.into())),
        screenshot,
        args.iter().any(|a| a == "--profile-editor"),
        None,
    )
}

fn prepare_editor(editor: &mut editor::Editor) {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|a| a == "--screenshot-controls") {
        settings_ui::open_controls(editor);
    }
    if let Some(relative) = args
        .windows(2)
        .find(|a| a[0] == "--inspect-asset")
        .map(|a| a[1].as_str())
    {
        match assets::inside(&editor.root, relative) {
            Ok(path) if path.exists() => project_browser::State::select_asset(editor, &path),
            Ok(_) => editor.log("Inspector file does not exist"),
            Err(error) => editor.log(error),
        }
    }
    if args
        .iter()
        .any(|a| a == "--screenshot-imports" || a == "--screenshot-import-dialog")
    {
        editor.assets.index = assets::scan(&editor.root, &mut Default::default());
        let capture_asset = args
            .windows(2)
            .find(|a| a[0] == "--inspect-asset")
            .and_then(|a| assets::inside(&editor.root, &a[1]).ok());
        let selected_record = editor
            .assets
            .index
            .usable()
            .find(|r| {
                capture_asset
                    .as_ref()
                    .map_or(r.meta.kind == assets::Kind::AudioClip, |path| {
                        &r.path == path
                    })
            })
            .cloned();
        if let Some(record) = selected_record {
            editor.assets.selected = Some(record.meta.id);
            editor.assets.operation_path = assets::path_string(&editor.root, &record.path);
            editor.assets.focus_tab = Some(2);
            editor.assets.window = true;
            if args.iter().any(|a| a == "--screenshot-import-dialog") {
                editor.assets.begin_reimport(&record, true);
            }
        }
    }
    if args.iter().any(|a| a == "--screenshot-blockout") {
        editor.assets.index = assets::scan(&editor.root, &mut Default::default());
        let record = editor
            .assets
            .index
            .usable()
            .find(|r| r.meta.kind == assets::Kind::EditableMesh)
            .cloned();
        if let Some(record) = record {
            mesh_editor::open(editor, record, None);
            editor.selected = editor.mesh_editor.target;
        }
    }
    if args.iter().any(|a| a == "--screenshot-terrain") {
        editor.assets.index = assets::scan(&editor.root, &mut Default::default());
        let record = editor
            .assets
            .index
            .usable()
            .find(|r| r.meta.kind == assets::Kind::Terrain)
            .cloned();
        match record {
            Some(record) => {
                terrain_editor::open(editor, record, None);
                editor.selected = editor.terrain_editor.target;
            }
            // Nothing to open yet: make one, so the capture always has a
            // terrain to show rather than an empty panel.
            None => {
                if let Err(error) = terrain_editor::create(editor) {
                    editor.log(error);
                }
            }
        }
    }
    if args.iter().any(|a| a == "--screenshot-inspector") {
        editor.selected = editor
            .scene
            .actors
            .iter()
            .enumerate()
            .max_by_key(|(_, actor)| actor.components.len())
            .map(|(index, _)| index)
            .or(editor.selected);
    }
    if args.iter().any(|a| a == "--screenshot-lighting") {
        editor.lighting_window = true;
        editor.selected = editor.scene.actors.iter().position(|e| e.light.is_some());
    }
    if args.iter().any(|a| a == "--screenshot-navigation") {
        editor.selected = (0..editor.scene.actors.len())
            .find(|&i| navigation::selected_volume(&editor.scene, Some(i)).is_some());
    }
    if args.iter().any(|a| a == "--screenshot-hud") {
        editor.set_scene_2d(true);
        editor.selected = editor
            .scene
            .actors
            .iter()
            .position(|e| e.progress.is_some())
            .or_else(|| editor.scene.actors.iter().position(|e| e.rect.is_some()))
            .or(editor.selected);
    }
    if args.iter().any(|a| a == "--screenshot-native-hud") {
        editor.set_scene_2d(true);
    }
    if args.iter().any(|a| a == "--screenshot-2d") {
        editor.set_scene_view_mode(scene_view_mode::SceneViewMode::TwoD);
    }
    if args.iter().any(|a| a == "--screenshot-package-disc") {
        export_ui::open(editor);
    }
    if args.iter().any(|a| a == "--screenshot-script-dialog") {
        editor.action("new-script");
        editor.script_name = "Boss".into();
        editor.script_folder = "Enemies".into();
        editor.script_parent = editor
            .catalog
            .iter()
            .find(|c| c.name == "Enemy")
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "ActorComponent".into());
    }
    if args.iter().any(|a| a == "--screenshot-native-game") {
        editor.play_profile.runtime = play::Runtime::NativePc;
        editor.play_profile.content = play::Content::CurrentScene;
        editor.build(true);
    } else if args.iter().any(|a| a == "--screenshot-game") {
        editor.build(true);
    }
}
