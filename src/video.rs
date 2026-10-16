//! Build staging for Video assets that Blueprints and Lua classes reference:
//! cooked `.STR` streams for the disc and the `video-bank.hh` table the PSX
//! player reads.
use crate::{assets, playback_staging::ResourceOutput};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use uuid::Uuid;

/// Disc names of the streams the last staging selected, one per line.
const LIST: &str = "video/streams.txt";
/// Two 16-pixel columns of decoded pixels, at the tallest picture.
const SLICE_BYTES: u32 = 2 * 16 * 240 * 2;

pub fn file_name(index: usize) -> String {
    format!("V{index:07}.STR")
}
pub fn is_staged_path(path: &str) -> bool {
    path.strip_prefix("video/V")
        .and_then(|p| p.strip_suffix(".STR"))
        .is_some_and(|n| n.len() == 7 && n.bytes().all(|b| b.is_ascii_digit()))
}
/// The streams the disc must carry, in clip order.
pub fn staged(build: &Path) -> Vec<String> {
    fs::read_to_string(build.join(LIST))
        .map(|text| text.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// Cooks every referenced clip, writes `video/V#######.STR` and
/// `video-bank.hh`. Without `include_streams` (Serial) clip indices survive
/// but nothing is staged for the disc, and playing does nothing.
pub fn stage(
    root: &Path,
    build: &Path,
    videos: &BTreeSet<Uuid>,
    index: &assets::Index,
    include_streams: bool,
) -> Result<Vec<ResourceOutput>, String> {
    let mut outputs = vec![];
    let mut inputs = BTreeMap::new();
    let mut clips = vec![];
    let mut ids = vec![];
    let mut names = vec![];
    let mut memory = 0;
    for (i, id) in videos.iter().enumerate() {
        let record = index.resolve(*id)?;
        let package = assets::Package::load(&record.path)?;
        if package.meta.id != *id
            || assets::cache_key(&package.meta) != assets::cache_key(&record.meta)
        {
            return Err(format!(
                "Video {id} changed after resource selection; retry staging"
            ));
        }
        if package.meta.kind != assets::Kind::Video {
            return Err(format!(
                "Asset {id} is {:?}, expected Video",
                package.meta.kind
            ));
        }
        inputs.insert(format!("asset:{id}"), assets::cache_key(&package.meta));
        ids.push(format!(
            "UINT64_C({})",
            crate::blueprint_refs::compact_id(&id.to_string())
        ));
        if !include_streams {
            clips.push("{}".to_string());
            continue;
        }
        let cooked = crate::video_import::cook(root, &package, || false)
            .map_err(|e| format!("Video {}: {e}", package.meta.source))?;
        let cook_key = assets::cook_key(root, &package.meta)?;
        inputs.insert(format!("video-cook:{id}"), cook_key.clone());
        let info = &cooked.report.info;
        let name = file_name(i);
        crate::project::write_changed(&build.join("video").join(&name), &cooked.stream)?;
        outputs.push(ResourceOutput {
            path: format!("video/{name}"),
            signature: assets::hash(&cooked.stream),
            inputs: BTreeMap::from([
                (format!("asset:{id}"), assets::cache_key(&package.meta)),
                (format!("video-cook:{id}"), cook_key),
            ]),
        });
        let flags = u8::from(cooked.report.tv_levels) | u8::from(info.audio.is_some()) << 1;
        clips.push(format!(
            "{{\"{name};1\",{},{},{},{},{},{flags},{},{}}}",
            info.width,
            info.height,
            info.frames,
            info.frame_bytes(),
            info.mdec_capacity(),
            info.xa_file,
            info.xa_channel
        ));
        memory = memory.max(2 * info.frame_bytes() + 4 * info.mdec_capacity());
        names.push(name);
    }
    let mut header = String::from(
        "// Generated from UUID asset references.\n#pragma once\n#include <stddef.h>\n#include <stdint.h>\n",
    );
    if clips.is_empty() || names.is_empty() {
        header.push_str("namespace epok {\ninline constexpr size_t video_clip_count = 0;\n");
    } else {
        header.push_str(&format!(
            "#define EPOK_HAS_VIDEO 1\n#include \"fmv.hpp\"\nnamespace epok {{\ninline constexpr uint32_t video_memory_bytes = {};\ninline constexpr VideoClip video_clips[] = {{{}}};\ninline constexpr size_t video_clip_count = {};\n",
            memory + SLICE_BYTES,
            clips.join(","),
            clips.len()
        ));
    }
    header.push_str(&format!(
        "namespace bp {{inline int video_index(uint64_t id) {{constexpr uint64_t ids[] = {{{}}};for (size_t i = 0; i < {}; ++i) if (ids[i] == id) return int(i);return -1;}}}}\n}}\n",
        if ids.is_empty() { "0".to_string() } else { ids.join(",") },
        ids.len()
    ));
    crate::project::write_changed(&build.join(LIST), names.join("\n").as_bytes())?;
    crate::project::write_changed(&build.join("video-bank.hh"), header.as_bytes())?;
    outputs.push(ResourceOutput {
        path: "video-bank.hh".into(),
        signature: assets::hash(header.as_bytes()),
        inputs,
    });
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    #[test]
    fn staged_names_and_paths_are_strict() {
        assert_eq!(super::file_name(12), "V0000012.STR");
        assert!(super::is_staged_path("video/V0000012.STR"));
        assert!(!super::is_staged_path("video/V12.STR"));
        assert!(!super::is_staged_path("music/M0000000.XA"));
        let build = std::env::temp_dir().join(format!("epok-video-{}", uuid::Uuid::new_v4()));
        let index = crate::assets::Index::default();
        super::stage(&build, &build, &Default::default(), &index, true).unwrap();
        let header = std::fs::read_to_string(build.join("video-bank.hh")).unwrap();
        assert!(header.contains("video_clip_count = 0"));
        assert!(!header.contains("EPOK_HAS_VIDEO"));
        assert!(super::staged(&build).is_empty());
        // The unique temporary directory created by this test owns all files.
        std::fs::remove_dir_all(build).unwrap();
    }
}
