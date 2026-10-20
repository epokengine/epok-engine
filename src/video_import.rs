//! Video assets. A package keeps the original source; cooking encodes it to a
//! PSX `.STR` stream with psxavenc and validates every frame with the
//! console's own decoder before anything reaches a disc.
use crate::{assets, fmv};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::Command, time::Duration};
use uuid::Uuid;

pub const IMPORTER_VERSION: u32 = 1;
pub const MAX_SOURCE: usize = 256 * 1024 * 1024;
/// Containers psxavenc reads through FFmpeg.
pub const EXTENSIONS: [&str; 11] = [
    "mp4", "m4v", "mov", "avi", "mkv", "webm", "mpg", "mpeg", "wmv", "ogv", "gif",
];
const ENCODE_TIMEOUT: Duration = Duration::from_secs(30 * 60);
/// Frames sampled to detect TV levels; enough to see a clip's darkest and
/// brightest shots without decoding minutes of video in the editor.
const LEVEL_SAMPLES: usize = 48;

pub fn is_source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioTrack {
    #[default]
    Stereo,
    Mono,
    None,
}
impl AudioTrack {
    pub const ALL: [Self; 3] = [Self::Stereo, Self::Mono, Self::None];
    pub fn label(self) -> &'static str {
        match self {
            Self::Stereo => "Stereo",
            Self::Mono => "Mono",
            Self::None => "No audio (all bandwidth to video)",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    V2,
    #[default]
    V3,
}
impl Codec {
    pub const ALL: [Self; 2] = [Self::V3, Self::V2];
    pub fn label(self) -> &'static str {
        match self {
            Self::V3 => "BS v3 (sharper at the same size)",
            Self::V2 => "BS v2 (widest decoder compatibility)",
        }
    }
    fn argument(self) -> &'static str {
        match self {
            Self::V2 => "v2",
            Self::V3 => "v3",
        }
    }
}
/// Brightness range of the source. Most camera and web video stores luma in
/// 16..235; the console shows such clips with grey blacks unless expanded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Levels {
    #[default]
    Auto,
    Full,
    Tv,
}
impl Levels {
    pub const ALL: [Self; 3] = [Self::Auto, Self::Full, Self::Tv];
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto (detect from the encoded frames)",
            Self::Full => "Full range (0-255)",
            Self::Tv => "TV range (16-235): expand on the console",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    /// The picture is fitted inside this box, keeping its aspect ratio
    /// unless `stretch` is set. Multiples of 16.
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    pub stretch: bool,
    pub audio: AudioTrack,
    pub sample_rate: u32,
    pub codec: Codec,
    pub levels: Levels,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            width: 320,
            height: 240,
            frame_rate: 15,
            stretch: false,
            audio: AudioTrack::Stereo,
            sample_rate: 37800,
            codec: Codec::V3,
            levels: Levels::Auto,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported video importer settings version".into());
        }
        if !(32..=320).contains(&self.width) || !self.width.is_multiple_of(16) {
            return Err("Video width must be 32 to 320 pixels in steps of 16".into());
        }
        if !(32..=240).contains(&self.height) || !self.height.is_multiple_of(16) {
            return Err("Video height must be 32 to 240 pixels in steps of 16".into());
        }
        if !(5..=30).contains(&self.frame_rate) {
            return Err("Video frame rate must be 5 to 30 frames per second".into());
        }
        if ![37800, 18900].contains(&self.sample_rate) {
            return Err("Video audio must use 37800 Hz or 18900 Hz".into());
        }
        Ok(())
    }
    /// One XA sector in every `interleave()` carries audio at double speed.
    pub fn interleave(&self) -> u32 {
        let base = if self.sample_rate == 37800 { 8 } else { 16 };
        match self.audio {
            AudioTrack::Stereo => base,
            AudioTrack::Mono => base * 2,
            AudioTrack::None => 1,
        }
    }
    /// Compressed bytes available to each video frame.
    pub fn frame_budget(&self) -> u32 {
        let interleave = self.interleave();
        let video = if interleave == 1 {
            fmv::SECTORS_PER_SECOND as f32
        } else {
            fmv::SECTORS_PER_SECOND as f32 * (interleave - 1) as f32 / interleave as f32
        };
        (video / self.frame_rate as f32 * fmv::CHUNK as f32) as u32
    }
}

/// The measured result of a cook, kept beside the stream for the Inspector.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub info: fmv::StreamInfo,
    pub frame_rate: u32,
    pub tv_levels: bool,
    /// Luma percentiles (0.2 % and 99.8 %) of the sampled frames.
    pub luma: (u8, u8),
    pub bytes: u64,
    /// Display size read from the source container, when it could be read.
    #[serde(default)]
    pub source_size: Option<(u32, u32)>,
}
pub struct Cooked {
    pub stream: Vec<u8>,
    pub report: Report,
}

fn implementation() -> &'static str {
    static IMPLEMENTATION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    IMPLEMENTATION.get_or_init(|| {
        assets::hash(
            [
                include_bytes!("video_import.rs").as_slice(),
                include_bytes!("fmv.rs").as_slice(),
                include_bytes!("../runtime/fmv_bitstream.hpp").as_slice(),
            ]
            .concat()
            .as_slice(),
        )
    })
}
/// Cook identity: settings, source, this implementation and the encoder binary.
pub fn cook_key(root: &Path, meta: &assets::Metadata) -> Result<String, String> {
    let tool = crate::disc::tool(root, "psxavenc")?;
    let tool = crate::dependencies::find_executable(&tool.to_string_lossy())
        .ok_or("Selected psxavenc executable disappeared")?;
    let encoder = assets::hash(&fs::read(tool).map_err(|e| e.to_string())?);
    Ok(assets::hash(
        format!(
            "psx-video:{}:{}:{encoder}",
            assets::cache_key(meta),
            implementation()
        )
        .as_bytes(),
    ))
}
/// Last-cook report for the Inspector, found without hashing the encoder.
pub fn last_report(root: &Path, meta: &assets::Metadata) -> Option<Report> {
    let path = root
        .join(".epok/imported")
        .join(assets::cache_key(meta))
        .join("Import.epokcache");
    crate::document::from_slice(&fs::read(path).ok()?).ok()
}
pub fn thumbnail_path(root: &Path, meta: &assets::Metadata) -> std::path::PathBuf {
    root.join(".epok/imported")
        .join(assets::cache_key(meta))
        .join("thumbnail.png")
}

/// The cooked stream for a package, encoding it on a cache miss.
pub fn cook(
    root: &Path,
    package: &assets::Package,
    cancelled: impl FnMut() -> bool,
) -> Result<Cooked, String> {
    let settings = package.meta.settings.video()?;
    settings.validate()?;
    let dir = root
        .join(".epok/imported")
        .join(cook_key(root, &package.meta)?);
    let path = dir.join("video.str");
    if let (Ok(stream), Ok(checksum), Ok(report)) = (
        fs::read(&path),
        fs::read_to_string(dir.join("checksum")),
        fs::read(dir.join("Import.epokcache")),
    ) && assets::hash(&stream) == checksum
        && let Ok(report) = crate::document::from_slice::<Report>(&report)
    {
        return Ok(Cooked { stream, report });
    }
    let extension = Path::new(&package.meta.source)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4")
        .to_ascii_lowercase();
    let source_size = crate::video_probe::display_size(&package.source);
    let picture = match source_size {
        Some(size) if !settings.stretch => {
            crate::video_probe::fit(size, settings.width, settings.height)
        }
        _ => (settings.width, settings.height),
    };
    let raw = encode(
        root,
        &package.source,
        &extension,
        settings,
        picture,
        cancelled,
    )?;
    let mut stream = fmv::parse(&raw)?;
    let Measured {
        tv_levels,
        luma,
        thumbnail,
    } = levels(&stream, settings.levels)?;
    stream.validate(tv_levels)?;
    let info = stream.info.clone();
    let data = fmv::finish(raw, &info);
    let report = Report {
        bytes: data.len() as u64,
        info,
        frame_rate: settings.frame_rate,
        tv_levels,
        luma,
        source_size,
    };
    let encoded = crate::document::to_vec(&report).map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    assets::replace_cache(&path, &data)?;
    assets::replace_cache(&dir.join("checksum"), assets::hash(&data).as_bytes())?;
    assets::replace_cache(&dir.join("Import.epokcache"), &encoded)?;
    let summary = root
        .join(".epok/imported")
        .join(assets::cache_key(&package.meta));
    fs::create_dir_all(&summary).map_err(|e| e.to_string())?;
    assets::replace_cache(&summary.join("Import.epokcache"), &encoded)?;
    if let Some(picture) = thumbnail {
        assets::replace_cache(&summary.join("thumbnail.png"), &png(&picture)?)?;
    }
    Ok(Cooked {
        stream: data,
        report,
    })
}

fn encode(
    root: &Path,
    source: &[u8],
    extension: &str,
    settings: &Settings,
    picture: (u32, u32),
    cancelled: impl FnMut() -> bool,
) -> Result<Vec<u8>, String> {
    let job = root
        .join(".epok/video-jobs")
        .join(Uuid::new_v4().to_string());
    fs::create_dir_all(&job).map_err(|e| e.to_string())?;
    let input = job.join(format!("input.{extension}"));
    let output = job.join("encoded.str");
    let result = (|| {
        fs::write(&input, source).map_err(|e| e.to_string())?;
        let mut command = Command::new(crate::disc::tool(root, "psxavenc")?);
        let silent = settings.audio == AudioTrack::None;
        command.args(["-q", "-t", if silent { "strv" } else { "str" }]);
        if !silent {
            let channels = if settings.audio == AudioTrack::Stereo {
                "2"
            } else {
                "1"
            };
            command.args(["-f", &settings.sample_rate.to_string(), "-c", channels]);
            command.args(["-b", "4", "-F", "1", "-C", "0"]);
        }
        command.args(["-v", settings.codec.argument()]);
        // Always an exact size: psxavenc 0.3.1's own aspect fitting encodes
        // the scaled picture with the unscaled dimensions.
        command.args(["-s", &format!("{}x{}", picture.0, picture.1), "-I"]);
        command.args(["-r", &settings.frame_rate.to_string(), "-x", "2"]);
        command.arg(&input).arg(&output);
        crate::disc::run_for(
            &mut command,
            &job.join("encode.log"),
            cancelled,
            ENCODE_TIMEOUT,
        )?;
        let encoded = fs::read(&output).map_err(|e| e.to_string())?;
        if silent {
            Ok(wrap_video_sectors(&encoded)?)
        } else {
            Ok(encoded)
        }
    })();
    // A freshly created importer-owned directory, never a source path.
    let _ = fs::remove_dir_all(&job);
    result
}
/// `strv` writes bare 2048-byte video sectors. Give each the Mode 2 Form 1
/// subheader the disc builder expects; mkpsxiso computes EDC/ECC from it.
fn wrap_video_sectors(sectors: &[u8]) -> Result<Vec<u8>, String> {
    if sectors.is_empty() || !sectors.len().is_multiple_of(2048) {
        return Err("Encoder returned incomplete video sectors".into());
    }
    let mut stream = Vec::with_capacity(sectors.len() / 2048 * fmv::SECTOR);
    for sector in sectors.chunks_exact(2048) {
        stream.extend_from_slice(&[1, 0, 0x48, 0, 1, 0, 0x48, 0]);
        stream.extend_from_slice(sector);
        stream.resize(stream.len() + fmv::SECTOR - 8 - 2048, 0);
    }
    Ok(stream)
}

/// The brightness mapping chosen for a stream, its measured levels and a
/// representative frame for the thumbnail.
struct Measured {
    tv_levels: bool,
    luma: (u8, u8),
    thumbnail: Option<fmv::Picture>,
}
/// Picks the brightness mapping and a representative thumbnail.
fn levels(stream: &fmv::Stream, choice: Levels) -> Result<Measured, String> {
    let (width, height) = (stream.info.width, stream.info.height);
    let mdec = fmv::Mdec::new(false);
    let mut histogram = [0_u64; 256];
    let step = stream.frames.len().div_ceil(LEVEL_SAMPLES).max(1);
    // A frame a fifth of the way in avoids black lead-ins as the thumbnail.
    let thumbnail_frame = stream.frames.len() / 5;
    let mut thumbnail = None;
    for (index, frame) in stream.frames.iter().enumerate() {
        if index % step != 0 && index != thumbnail_frame {
            continue;
        }
        let words = fmv::decode(&frame.data, width, height, false)
            .map_err(|e| format!("Frame {}: {e}", frame.number))?;
        let picture = mdec.picture(&words, width, height)?;
        for (total, count) in histogram.iter_mut().zip(picture.histogram.iter()) {
            *total += *count as u64;
        }
        if index == thumbnail_frame {
            thumbnail = Some(picture);
        }
    }
    let total: u64 = histogram.iter().sum();
    let percentile = |fraction: f64| {
        let target = (total as f64 * fraction) as u64;
        let mut seen = 0;
        histogram
            .iter()
            .position(|count| {
                seen += count;
                seen > target
            })
            .unwrap_or(255) as u8
    };
    let luma = (percentile(0.002), percentile(0.998));
    // Limited-range video puts its blacks near 16 and its whites near 235.
    // Clips that reach neither say nothing about their range: leave them alone.
    let detected = (12..=20).contains(&luma.0) && (228..=240).contains(&luma.1);
    let tv_levels = match choice {
        Levels::Auto => detected,
        Levels::Full => false,
        Levels::Tv => true,
    };
    if tv_levels && let Some(picture) = &mut thumbnail {
        // Show the thumbnail the way the console will.
        let frame = &stream.frames[thumbnail_frame];
        let words = fmv::decode(&frame.data, width, height, true)?;
        *picture = fmv::Mdec::new(true).picture(&words, width, height)?;
    }
    Ok(Measured {
        tv_levels,
        luma,
        thumbnail,
    })
}

pub fn png(picture: &fmv::Picture) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut encoder = ::png::Encoder::new(&mut bytes, picture.width, picture.height);
    encoder.set_color(::png::ColorType::Rgba);
    encoder.set_depth(::png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer
        .write_image_data(&picture.rgba)
        .map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// A package's source is checked by the encoder; here only its presence.
pub fn probe(source: &[u8]) -> Result<(), String> {
    if source.is_empty() {
        return Err("Empty video source".into());
    }
    Ok(())
}

pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    settings: Settings,
    existing: Option<&assets::Record>,
    snapshot: bool,
) -> Result<assets::Candidate, String> {
    settings.validate()?;
    if existing.is_some_and(|r| r.meta.kind != assets::Kind::Video) {
        return Err("Reimport requires a Video asset".into());
    }
    let dest = assets::inside(root, destination)?;
    if dest.extension().is_none_or(|x| x != "epokasset") {
        return Err("Video destination must end in .epokasset".into());
    }
    let old = existing
        .map(|r| assets::Package::load(&r.path))
        .transpose()?;
    let path = if snapshot {
        None
    } else {
        Some(assets::inside(root, source)?)
    };
    let bytes = if let Some(p) = &path {
        assets::read_video_bounded(p)?
    } else {
        old.as_ref().ok_or("Missing video snapshot")?.source.clone()
    };
    let hash = assets::hash(&bytes);
    let package = assets::Package {
        meta: assets::Metadata {
            version: 2,
            id: existing.map_or_else(Uuid::new_v4, |r| r.meta.id),
            kind: assets::Kind::Video,
            importer_version: IMPORTER_VERSION,
            source: if snapshot {
                old.as_ref().unwrap().meta.source.clone()
            } else {
                source.replace('\\', "/")
            },
            source_hash: hash.clone(),
            settings: crate::import_settings::Settings::Video(settings),
            extra: old.map(|p| p.meta.extra).unwrap_or_default(),
        },
        source: bytes,
    };
    // Encode now: a source the console cannot play never becomes an asset, and
    // the Inspector opens on a warm cache.
    cook(root, &package, || false)?;
    Ok(assets::Candidate {
        destination: dest,
        expected: existing.map(|r| r.revision.clone()),
        source_path: path,
        source_hash: hash,
        package,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_validate_bandwidth_and_round_trip() {
        let settings = Settings::default();
        settings.validate().unwrap();
        assert_eq!(settings.interleave(), 8);
        // 150 sectors/s, 7 of 8 carry video, 15 frames/s.
        assert_eq!(settings.frame_budget(), 17640);
        let silent = Settings {
            audio: AudioTrack::None,
            ..Default::default()
        };
        assert_eq!(silent.frame_budget(), 20160);
        for invalid in [
            Settings {
                width: 330,
                ..Default::default()
            },
            Settings {
                height: 250,
                ..Default::default()
            },
            Settings {
                frame_rate: 60,
                ..Default::default()
            },
            Settings {
                sample_rate: 44100,
                ..Default::default()
            },
        ] {
            assert!(invalid.validate().is_err());
        }
        let wrapped = crate::import_settings::Settings::Video(Settings {
            levels: Levels::Tv,
            codec: Codec::V2,
            ..Default::default()
        });
        let text = serde_json::to_string(&wrapped).unwrap();
        assert!(text.contains("\"type\":\"Video\""));
        let parsed: crate::import_settings::Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, wrapped);
        assert!(
            serde_json::from_str::<crate::import_settings::Settings>(
                r#"{"type":"Video","options":{"width":31}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn silent_video_sectors_get_form_one_subheaders() {
        let wrapped = wrap_video_sectors(&[7_u8; 4096]).unwrap();
        assert_eq!(wrapped.len(), 2 * fmv::SECTOR);
        assert_eq!(&wrapped[..8], &[1, 0, 0x48, 0, 1, 0, 0x48, 0]);
        assert_eq!(wrapped[8], 7);
        assert!(wrap_video_sectors(&[0; 100]).is_err());
    }

    #[test]
    fn extensions_route_video_sources() {
        assert!(is_source("assets/Intro.MP4"));
        assert!(is_source("assets/cut/scene.webm"));
        assert!(!is_source("assets/theme.mp3"));
    }
}
