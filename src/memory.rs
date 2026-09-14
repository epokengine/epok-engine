//! Build-time memory accounting. Runtime heap/stack peaks are deliberately unknown.
//! The staging manifest names the resources actually selected by the Play pipeline;
//! the linked ELF supplies allocation sizes (including NOLOAD/BSS), never source sizes.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

const RAM: u64 = 2 * 1024 * 1024;
const MANIFEST: &str = "memory-inputs.json";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub bytes: u64,
    pub detail: String,
    pub asset: Option<String>,
    pub scenes: Vec<String>,
    pub children: Vec<Node>,
}
impl Node {
    pub fn leaf(name: impl Into<String>, bytes: u64, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bytes,
            detail: detail.into(),
            ..Default::default()
        }
    }
    pub fn group(name: impl Into<String>, mut children: Vec<Node>) -> Self {
        children.retain(|n| n.bytes != 0);
        children.sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.name.cmp(&b.name)));
        Self {
            bytes: children.iter().map(|n| n.bytes).sum(),
            name: name.into(),
            children,
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Space {
    pub capacity: Option<u64>,
    pub used: u64,
    pub root: Node,
}
impl Space {
    fn new(name: &str, capacity: Option<u64>, children: Vec<Node>, remainder: &str) -> Self {
        let mut root = Node::group(name, children);
        let used = root.bytes;
        if let Some(capacity) = capacity
            && capacity > used
        {
            root.children
                .push(Node::leaf(remainder, capacity - used, remainder));
            root.bytes = capacity;
        }
        Self {
            capacity,
            used,
            root,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VramRect {
    pub name: String,
    pub rect: [u16; 4],
    pub category: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneReport {
    pub name: String,
    pub vram: Space,
    pub rectangles: Vec<VramRect>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub profile: crate::play::Profile,
    pub debug: bool,
    pub executable_hash: String,
    pub ram: Space,
    pub scratchpad: Space,
    pub scenes: Vec<SceneReport>,
    pub spu: Space,
    pub files: Space,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Hint {
    token: String,
    category: String,
    resource: Node,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Manifest {
    hints: Vec<Hint>,
    scenes: Vec<SceneReport>,
    spu: Vec<Node>,
    files: Vec<Node>,
    #[serde(default)]
    warnings: Vec<String>,
}
pub fn build_warnings(build: &Path) -> Result<Vec<String>, String> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(build.join(MANIFEST)).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(manifest.warnings)
}
pub fn external_file_bytes(build: &Path) -> Result<u64, String> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(build.join(MANIFEST)).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(manifest.files.iter().map(|node| node.bytes).sum())
}
pub fn external_file_paths(build: &Path) -> Result<Vec<String>, String> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(build.join(MANIFEST)).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    manifest
        .files
        .iter()
        .map(|node| {
            if node.name == "GEOMETRY.BIN" {
                return Ok(node.name.clone());
            }
            node.name
                .rsplit_once(" (")
                .and_then(|(_, path)| path.strip_suffix(')'))
                .filter(|path| {
                    path.strip_prefix("music/M")
                        .and_then(|p| p.strip_suffix(".XA"))
                        .is_some_and(|n| n.len() == 7 && n.bytes().all(|b| b.is_ascii_digit()))
                })
                .map(str::to_owned)
                .ok_or_else(|| "Unrecognized external report payload".into())
        })
        .collect()
}

/// Capture names/layouts alongside the generated headers, inside the existing
/// staging transaction. No second asset selection or directory-wide size scan.
#[allow(clippy::too_many_arguments)]
pub fn stage(
    root: &Path,
    build: &Path,
    banks: &[crate::scene::Scene],
    shared: &crate::scene::Scene,
    index: &crate::assets::Index,
    global_layout: bool,
    catalog: &[crate::scripts::Script],
    audio_outputs: &[crate::playback_staging::ResourceOutput],
) -> Result<(), String> {
    let resource = |id: uuid::Uuid, bytes: u64, detail: &str| -> Result<Node, String> {
        let record = index.resolve(id)?;
        let mut node = Node::leaf(&record.meta.source, bytes, detail);
        node.asset = Some(crate::assets::path_string(root, &record.path));
        node.scenes = banks
            .iter()
            .filter(|s| {
                crate::texture::ids(s).contains(&id) || crate::audio::clip_ids(s).contains(&id)
            })
            .map(|s| s.name.clone())
            .collect();
        Ok(node)
    };
    let mut hints = Vec::new();
    for (i, id) in crate::texture::ids(shared).into_iter().enumerate() {
        for suffix in ["pixels", "palette"] {
            hints.push(Hint {
                token: format!("epok::texture_{i}_{suffix}"),
                category: "Textures".into(),
                resource: resource(
                    id,
                    0,
                    "Resident source data; counted once across scene banks",
                )?,
            });
        }
    }
    let mut spu = vec![Node::leaf(
        "Capture / system reserve",
        4096,
        "Runtime starts samples at SPU address 4096; any reverb work area is accounted separately",
    )];
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut sequence_banks = std::collections::BTreeSet::new();
    let mut sequence_reverb = false;
    for (i, id) in crate::audio::clip_ids(shared).into_iter().enumerate() {
        let sequence_path = format!("audio/{id}.epsq");
        if audio_outputs.iter().any(|o| o.path == sequence_path) {
            let bytes = fs::read(build.join(&sequence_path)).map_err(|e| e.to_string())?;
            if bytes.len() < 40 || &bytes[..4] != b"EPSQ" {
                return Err("Invalid staged PSX sequence in memory report".into());
            }
            let limit = u16::from_le_bytes([bytes[10], bytes[11]]);
            let bank_id = uuid::Uuid::from_slice(&bytes[24..40]).map_err(|e| e.to_string())?;
            hints.push(Hint { token: format!("epok::sequence_data_{i}"), category: "Audio".into(), resource: resource(id, bytes.len() as u64, &format!("Resident sequence, {limit} voice ceiling; events share the 24 physical voices with SFX"))? });
            hints.push(Hint { token: format!("epok::sequence_prepared_{i}"), category: "Audio".into(), resource: resource(id, 0, "Immutable note-start states and lookup table, prepared before the audio clock starts; exact storage measured from the linked symbol")? });
            let bank_index = sequence_banks.len();
            if sequence_banks.insert(bank_id) {
                let path = format!("audio/{bank_id}.epsb");
                if !audio_outputs.iter().any(|o| o.path == path) {
                    return Err("Staged sequence is missing its SoundBank payload".into());
                }
                let bytes = fs::read(build.join(path)).map_err(|e| e.to_string())?;
                if bytes.len() < 32 || &bytes[..4] != b"EPSB" {
                    return Err("Invalid staged PSX bank in memory report".into());
                }
                let version = u16::from_le_bytes([bytes[4], bytes[5]]);
                let (data_field, source_id) = match version {
                    1 => (20, bank_id),
                    2 if bytes.len() >= 48 => {
                        let sequence = index.resolve(id)?;
                        let source = crate::sequence::resolve_bank(
                            root,
                            sequence.meta.settings.sequence()?,
                            index,
                        )?;
                        if !sequence_reverb
                            && u32::from_le_bytes(bytes[40..44].try_into().unwrap()) == 1
                        {
                            sequence_reverb = true;
                            spu.push(Node::leaf("Global Room reverb",crate::psx_music_settings::ROOM_REVERB_BYTES as u64,
                                "One shared SPU work area at 0x7d940; sequence ownership is exclusive"));
                        }
                        (24, source.meta.id)
                    }
                    _ => return Err("Unsupported staged PSX bank version in memory report".into()),
                };
                let data_offset =
                    u32::from_le_bytes(bytes[data_field..data_field + 4].try_into().unwrap())
                        as usize;
                if data_offset > bytes.len() {
                    return Err("Invalid staged PSX bank data offset".into());
                }
                hints.push(Hint { token: format!("epok::sequence_bank_data_{bank_index}"), category: "Audio".into(), resource: resource(source_id, bytes.len() as u64, &format!("Resident derived bank {bank_id}: metadata and DMA sample source; counted once for sequences sharing this derivation"))? });
                spu.push(resource(source_id, (bytes.len()-data_offset) as u64, &format!("SoundBank {bank_id} SPU samples, deduplicated within this bank and aligned to 64 bytes; shared with SFX budget"))?);
            }
            continue;
        }
        let path = format!("audio/{id}.adpcm");
        // Only outputs selected by this staging pass are eligible: old cached
        // clips or XA files left in the build folder must never be included.
        if audio_outputs.iter().any(|o| o.path == path) {
            let bytes = fs::metadata(build.join(&path))
                .map_err(|e| e.to_string())?
                .len();
            let node = resource(id, bytes, "Resident ADPCM, including 64-byte DMA alignment")?;
            hints.push(Hint {
                token: format!("epok::audio_data_{i}"),
                category: "Audio".into(),
                resource: node.clone(),
            });
            spu.push(node);
        } else {
            let path = format!("music/M{i:07}.XA");
            if audio_outputs.iter().any(|o| o.path == path) {
                let bytes = fs::metadata(build.join(&path))
                    .map_err(|e| e.to_string())?
                    .len();
                let mut node =
                    resource(id, bytes, "External XA stream; not a resident SPU sample")?;
                node.name = format!("{} ({path})", node.name);
                files.push(node);
            } else if index.resolve(id)?.meta.settings.audio()?.is_streamed() {
                warnings.push(format!("XA music omitted for Serial: {}. XA requires the physical CD decoder and cannot be embedded in the EXE or served by PCDrv. Playback calls for this clip do nothing; resident sound effects remain available.", index.resolve(id)?.meta.source));
            }
        }
    }
    if build.join("GEOMETRY.BIN").is_file() {
        files.push(Node::leaf(
            "GEOMETRY.BIN",
            fs::metadata(build.join("GEOMETRY.BIN"))
                .map_err(|e| e.to_string())?
                .len(),
            "External geometry archive; the RAM page pool is counted in the ELF",
        ));
    }
    for script in catalog {
        let mut node = Node::leaf(
            &script.name,
            0,
            "Linked script code/data; compiler inlining may attribute some bytes to its caller",
        );
        node.asset = Some(format!("assets/scripts/{}", script.header_path()));
        for class in &script.classes {
            hints.push(Hint {
                token: format!("{}::", class.cpp_name),
                category: "Scripts / Blueprints".into(),
                resource: node.clone(),
            });
        }
    }
    let mut scenes = Vec::new();
    for (bank, s) in banks.iter().enumerate() {
        let mut scene_node = Node::leaf(
            &s.name,
            0,
            "Prelinked scene bank; remains in main RAM even when another scene is active",
        );
        scene_node.scenes.push(s.name.clone());
        hints.push(Hint {
            token: format!("epok::scene_{bank}::"),
            category: "Scene data / geometry".into(),
            resource: scene_node,
        });
        let layout_scene = if global_layout { shared } else { s };
        let mut textures = Vec::new();
        let mut palettes = Vec::new();
        let mut rectangles = Vec::new();
        for (id, p) in crate::texture::layout(layout_scene)? {
            let t = &shared.textures[&id];
            let width = t.width.div_ceil(4) * 2;
            let node = resource(
                id,
                u64::from(width) * u64::from(t.height) * 2,
                "Active bank texture allocation (16-bit VRAM words)",
            )?;
            rectangles.push(VramRect {
                name: node.name.clone(),
                rect: [p.x, p.y, width, t.height],
                category: 1,
            });
            textures.push(node);
            let node = resource(id, 512, "256 palette entries, 16 bits each")?;
            rectangles.push(VramRect {
                name: format!("{} palette", node.name),
                rect: [640, p.clut_y, 256, 1],
                category: 2,
            });
            palettes.push(node);
        }
        let width = s.display_size[0];
        let framebuffer_bytes = u64::from(width) * 480 * 2;
        if s.display_size[1] == 480 {
            rectangles.push(VramRect {
                name: "Interlaced framebuffer".into(),
                rect: [0, 0, width, 480],
                category: 0,
            });
        } else {
            for y in [0, 240] {
                rectangles.push(VramRect {
                    name: format!("Framebuffer {}", y / 240),
                    rect: [0, y, width, 240],
                    category: 0,
                });
            }
        }
        if width < 640 {
            rectangles.push(VramRect {
                name: "Reserved display columns".into(),
                rect: [width, 0, 640 - width, 480],
                category: 3,
            });
        }
        rectangles.push(VramRect {
            name: "HUD / loading font".into(),
            rect: [960, 448, 64, 64],
            category: 3,
        });
        rectangles.push(VramRect {
            name: "Loading image reserve".into(),
            rect: [960, 384, 64, 64],
            category: 3,
        });
        let vram = Space::new(
            "VRAM",
            Some(1024 * 1024),
            vec![
                Node::leaf(
                    "Framebuffers",
                    framebuffer_bytes,
                    "Progressive: two 240-line buffers. Interlaced: one 480-line surface.",
                ),
                Node::group("Textures", textures),
                Node::group("Palettes", palettes),
                Node::group(
                    "Reserved layout",
                    vec![
                        Node::leaf(
                            "Display columns",
                            640 * 480 * 2 - framebuffer_bytes,
                            "The current texture allocator excludes the display region",
                        ),
                        Node::leaf(
                            "HUD / loading font",
                            64 * 64 * 2,
                            "Persistent font at (960,448)",
                        ),
                        Node::leaf(
                            "Loading image region",
                            64 * 64 * 2,
                            "Reserved at (960,384), including when no image is selected",
                        ),
                    ],
                ),
            ],
            "Unallocated (layout constrained)",
        );
        scenes.push(SceneReport {
            name: s.name.clone(),
            vram,
            rectangles,
        });
    }
    let manifest = Manifest {
        hints,
        scenes,
        spu,
        files,
        warnings,
    };
    crate::project::write_changed(
        &build.join(MANIFEST),
        &serde_json::to_vec(&manifest).map_err(|e| e.to_string())?,
    )
}

#[derive(Clone, Debug)]
struct Section {
    name: String,
    address: u64,
    size: u64,
    flags: u32,
    kind: u32,
}
#[derive(Clone, Debug)]
struct Symbol {
    name: String,
    address: u64,
    size: u64,
    section: usize,
}
struct Elf {
    sections: Vec<Section>,
    symbols: Vec<Symbol>,
}

fn bytes(data: &[u8], offset: u64, count: u64) -> Result<&[u8], String> {
    let end = offset.checked_add(count).ok_or("ELF range overflow")?;
    data.get(
        usize::try_from(offset).map_err(|_| "ELF offset overflow")?
            ..usize::try_from(end).map_err(|_| "ELF range overflow")?,
    )
    .ok_or("Truncated ELF".into())
}
fn u16_at(data: &[u8], offset: u64) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        bytes(data, offset, 2)?.try_into().unwrap(),
    ))
}
fn u32_at(data: &[u8], offset: u64) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        bytes(data, offset, 4)?.try_into().unwrap(),
    ))
}
fn string(data: &[u8], offset: u64) -> Result<String, String> {
    let tail = data
        .get(usize::try_from(offset).map_err(|_| "ELF string overflow")?..)
        .ok_or("Invalid ELF string offset")?;
    let end = tail
        .iter()
        .position(|v| *v == 0)
        .ok_or("Unterminated ELF string")?;
    Ok(String::from_utf8_lossy(&tail[..end]).into_owned())
}
impl Elf {
    fn parse(data: &[u8]) -> Result<Self, String> {
        if bytes(data, 0, 7)? != b"\x7fELF\x01\x01\x01"
            || u16_at(data, 16)? != 2
            || u16_at(data, 18)? != 8
        {
            return Err("Expected a linked 32-bit little-endian MIPS ELF".into());
        }
        let offset = u64::from(u32_at(data, 32)?);
        let stride = u64::from(u16_at(data, 46)?);
        let count = usize::from(u16_at(data, 48)?);
        let strings = usize::from(u16_at(data, 50)?);
        if stride < 40 || count == 0 || strings >= count {
            return Err("Unsupported ELF section table".into());
        }
        bytes(data, offset, stride * count as u64)?;
        let header = |i: usize| offset + stride * i as u64;
        let table = |i: usize| -> Result<&[u8], String> {
            let h = header(i);
            bytes(
                data,
                u64::from(u32_at(data, h + 16)?),
                u64::from(u32_at(data, h + 20)?),
            )
        };
        let names = table(strings)?;
        let mut sections = Vec::new();
        let mut symbols = Vec::new();
        for i in 0..count {
            let h = header(i);
            let kind = u32_at(data, h + 4)?;
            let section = Section {
                name: string(names, u64::from(u32_at(data, h)?))?,
                address: u64::from(u32_at(data, h + 12)?),
                size: u64::from(u32_at(data, h + 20)?),
                flags: u32_at(data, h + 8)?,
                kind,
            };
            // NOLOAD has a memory size but no corresponding file bytes.
            if kind != 8 {
                table(i)?;
            }
            sections.push(section);
            if kind == 2 {
                let link = u32_at(data, h + 24)? as usize;
                let entry = u64::from(u32_at(data, h + 36)?);
                if link >= count || entry < 16 {
                    return Err("Invalid ELF symbol table".into());
                }
                let names = table(link)?;
                let syms = table(i)?;
                if !(syms.len() as u64).is_multiple_of(entry) {
                    return Err("Truncated ELF symbol table".into());
                }
                for j in 0..syms.len() as u64 / entry {
                    let p = j * entry;
                    let section = usize::from(u16_at(syms, p + 14)?);
                    let symbol = Symbol {
                        name: string(names, u64::from(u32_at(syms, p)?))?,
                        address: u64::from(u32_at(syms, p + 4)?),
                        size: u64::from(u32_at(syms, p + 8)?),
                        section,
                    };
                    if !symbol.name.is_empty() {
                        symbols.push(symbol);
                    }
                }
            }
        }
        Ok(Self { sections, symbols })
    }
}

fn demangled_names(
    root: &Path,
    build: &Path,
    config: &crate::project::Config,
) -> BTreeMap<u64, String> {
    let tool = if config.toolchain_bin.is_empty() {
        std::path::PathBuf::from("mipsel-none-elf-nm")
    } else {
        crate::project::Config::path(root, &config.toolchain_bin).join(if cfg!(windows) {
            "mipsel-none-elf-nm.exe"
        } else {
            "mipsel-none-elf-nm"
        })
    };
    let mut command = std::process::Command::new(tool);
    command
        .args(["--defined-only", "--demangle", "--print-size"])
        .arg(build.join("epok.elf"));
    crate::pipeline::quiet(&mut command);
    let Ok(output) = command.output() else {
        return Default::default();
    };
    if !output.status.success() {
        return Default::default();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(4, ' ');
            let address = u64::from_str_radix(parts.next()?, 16).ok()?;
            let _size = parts.next()?;
            let _kind = parts.next()?;
            let name = parts.next()?.trim();
            (!name.is_empty()).then(|| (address, name.into()))
        })
        .collect()
}

fn category(name: &str, section: &Section) -> &'static str {
    if section.flags & 4 != 0 {
        "Engine / SDK code"
    } else if ["stream_pool", "page_pool", "streaming_cache"]
        .iter()
        .any(|s| name.contains(s))
    {
        "Streaming buffers"
    } else if ["texture_", "hud_font_pixels", "loading_pixels"]
        .iter()
        .any(|s| name.contains(s))
    {
        "Textures"
    } else if name.contains("audio_") || name.contains("sequence_") || name.contains("psx_audio::")
    {
        "Audio"
    } else if section.kind == 8 {
        "Pools / runtime state"
    } else {
        "Other resident data"
    }
}

fn symbol_matches(name: &str, token: &str) -> bool {
    let identifier = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == ':';
    name.match_indices(token).any(|(start, _)| {
        let before = name[..start].chars().next_back();
        let after = name[start + token.len()..].chars().next();
        before.is_none_or(|c| !identifier(c))
            && (token.ends_with("::") || after.is_none_or(|c| !identifier(c)))
    })
}

/// Divide section spans among symbols without counting aliases/overlaps twice.
/// Bytes with no sized symbol stay explicit instead of disappearing from totals.
fn section_nodes(
    section_index: usize,
    section: &Section,
    symbols: &[Symbol],
    hints: &[Hint],
    names: &BTreeMap<u64, String>,
) -> Vec<(String, Node)> {
    let mut selected = symbols
        .iter()
        .filter(|s| {
            s.section == section_index
                && s.size > 0
                && s.address >= section.address
                && s.address < section.address + section.size
        })
        .collect::<Vec<_>>();
    selected.sort_by(|a, b| {
        a.address
            .cmp(&b.address)
            .then(b.size.cmp(&a.size))
            .then(a.name.cmp(&b.name))
    });
    let mut cursor = section.address;
    let end = cursor + section.size;
    let mut result = Vec::new();
    for symbol in selected {
        if symbol.address > cursor {
            result.push((
                "Alignment / unattributed".into(),
                Node::leaf(
                    &section.name,
                    symbol.address - cursor,
                    format!("No sized symbol at 0x{cursor:08x}"),
                ),
            ));
        }
        let start = symbol.address.max(cursor);
        let stop = (symbol.address + symbol.size).min(end);
        if stop <= start {
            continue;
        }
        let name = names.get(&symbol.address).unwrap_or(&symbol.name);
        let hint = hints.iter().find(|h| symbol_matches(name, &h.token));
        let mut node = hint
            .map(|h| h.resource.clone())
            .unwrap_or_else(|| Node::leaf(name, 0, ""));
        let symbol_node = Node::leaf(
            name,
            stop - start,
            format!(
                "{} at 0x{start:08x} · {}",
                section.name,
                if section.kind == 8 {
                    "reserved at link time (not stored in EXE)"
                } else {
                    "resident linked bytes"
                }
            ),
        );
        node.bytes = stop - start;
        node.children = vec![symbol_node];
        result.push((
            hint.map(|h| h.category.clone())
                .unwrap_or_else(|| category(name, section).into()),
            node,
        ));
        cursor = stop;
    }
    if cursor < end {
        result.push((
            "Alignment / unattributed".into(),
            Node::leaf(
                &section.name,
                end - cursor,
                format!("No sized symbol at 0x{cursor:08x}"),
            ),
        ));
    }
    result
}

fn merge(nodes: Vec<(String, Node)>) -> Vec<Node> {
    let mut groups: BTreeMap<String, BTreeMap<(String, Option<String>), Node>> = BTreeMap::new();
    for (category, node) in nodes {
        let assets = groups.entry(category).or_default();
        let key = (node.name.clone(), node.asset.clone());
        if let Some(previous) = assets.get_mut(&key) {
            previous.bytes += node.bytes;
            previous.children.extend(node.children);
        } else {
            assets.insert(key, node);
        }
    }
    groups
        .into_iter()
        .map(|(name, assets)| Node::group(name, assets.into_values().collect()))
        .collect()
}

pub fn analyze(
    root: &Path,
    build: &Path,
    profile: crate::play::Profile,
    debug: bool,
    config: &crate::project::Config,
) -> Result<Report, String> {
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(build.join(MANIFEST)).map_err(|e| format!("Memory manifest: {e}"))?,
    )
    .map_err(|e| e.to_string())?;
    let elf =
        Elf::parse(&fs::read(build.join("epok.elf")).map_err(|e| format!("Memory ELF: {e}"))?)?;
    let names = demangled_names(root, build, config);
    let mut allocated = elf
        .sections
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.flags & 2 != 0
                && s.size > 0
                && s.name != ".PSX_EXE_Header"
                && s.address >= 0x80000000
                && s.address < 0x80800000
        })
        .collect::<Vec<_>>();
    allocated.sort_by_key(|(_, s)| s.address);
    if allocated.is_empty() {
        return Err("ELF has no main-RAM allocations".into());
    }
    let mut nodes = Vec::new();
    let mut cursor = 0x80000000u64;
    for (i, s) in allocated {
        if s.address < cursor {
            return Err("Overlapping main-RAM ELF sections; cannot provide an exact total".into());
        }
        if s.address > cursor {
            nodes.push((
                if cursor == 0x80000000 {
                    "System / load address"
                } else {
                    "Alignment / unattributed"
                }
                .into(),
                Node::leaf(
                    format!("0x{cursor:08x}..0x{:08x}", s.address),
                    s.address - cursor,
                    if cursor == 0x80000000 {
                        "Below the executable load address; unavailable to this build"
                    } else {
                        "Gap between linked sections"
                    },
                ),
            ));
        }
        nodes.extend(section_nodes(i, s, &elf.symbols, &manifest.hints, &names));
        cursor = s.address + s.size;
    }
    if let Some(heap) = elf.symbols.iter().find(|s| s.name == "__heap_start")
        && heap.address > cursor
    {
        nodes.push((
            "Alignment / unattributed".into(),
            Node::leaf(
                "Heap alignment",
                heap.address - cursor,
                "Gap from the final allocation to the linker's heap start",
            ),
        ));
    }
    let ram = Space::new(
        "Main RAM",
        Some(RAM),
        merge(nodes),
        "Unassigned (heap / stack)",
    );
    let scratch = elf
        .sections
        .iter()
        .enumerate()
        .filter(|(_, s)| s.flags & 2 != 0 && s.address >= 0x1f800000 && s.address < 0x1f800400)
        .flat_map(|(i, s)| section_nodes(i, s, &elf.symbols, &manifest.hints, &names))
        .collect();
    let scratchpad = Space::new(
        "Scratchpad",
        Some(1024),
        merge(scratch),
        "Unassigned scratchpad",
    );
    let exe = fs::read(build.join("epok.ps-exe")).map_err(|e| e.to_string())?;
    if !exe.starts_with(b"PS-X EXE") {
        return Err("Memory analysis requires a valid PS-X EXE".into());
    }
    let mut files = manifest.files;
    files.push(Node::leaf(
        "epok.ps-exe",
        exe.len() as u64,
        "File bytes including PS-X EXE header and padding; BSS is excluded",
    ));
    let mut warnings=vec!["Static allocations are measured from the linked ELF. Heap, stack and transition peaks are not measured; unassigned RAM is their remaining budget, not guaranteed free runtime memory.".into(),"Whole-game scene data stays prelinked. Shared resources and the reusable entity pool are counted once. VRAM is shown for each active bank.".into(),"File totals include the EXE and selected external payloads. Disc filesystem/sector overhead is not included.".into()];
    warnings.extend(manifest.warnings);
    if profile.target == crate::play::Target::Serial {
        warnings.push("The external Unirom/PCDrv handler's RAM reservation is not described by this ELF and is not measured here.".into());
    }
    if names.is_empty() {
        warnings.push("Demangled symbol names were unavailable. Exact section totals remain valid; resource attribution may be incomplete.".into());
    }
    if ram.used > RAM {
        warnings.push(format!(
            "Static RAM exceeds the retail PSX budget by {} bytes.",
            ram.used - RAM
        ));
    }
    let report = Report {
        profile,
        debug,
        executable_hash: crate::assets::hash(&exe),
        ram,
        scratchpad,
        scenes: manifest.scenes,
        spu: Space::new(
            "SPU audio RAM",
            Some(512 * 1024),
            manifest.spu,
            "Unassigned SPU RAM",
        ),
        files: Space::new("Build files", None, files, ""),
        warnings,
    };
    fs::write(
        build.join("memory-report.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staging_manifest_follows_unsaved_current_and_whole_game_inputs() {
        use crate::{
            play::{Content, DataSource, Profile, Target},
            scene::Scene,
        };
        let root = crate::workspace::tests::temp("memory-snapshots");
        let project =
            crate::workspace::create(&root, "Memory snapshots", crate::workspace::Template::Basic)
                .unwrap();
        let startup = crate::workspace::scene_path(&root, &project.manifest).unwrap();
        let mut other = Scene::load(&startup).unwrap();
        other.name = "Other".into();
        let other_path = root.join("assets/scenes/Other.epokmap");
        other.save(&other_path).unwrap();
        crate::scene_bank::Registry {
            scenes: vec!["assets/scenes/Other.epokmap".into()],
        }
        .save(&root)
        .unwrap();
        let saved = fs::read(&other_path).unwrap();
        other.name = "Unsaved other".into();
        for (content, count) in [(Content::CurrentScene, 1), (Content::WholeGame, 2)] {
            let profile = Profile {
                target: Target::Serial,
                content,
                data: DataSource::Executable,
                ..Default::default()
            };
            let input =
                crate::play::input(&root, other_path.clone(), other.clone(), profile, false)
                    .unwrap();
            let build = root.join(".epok/build");
            crate::project::stage_prepared(&root, &input, &input.scene, &build).unwrap();
            let manifest: Manifest =
                serde_json::from_slice(&fs::read(build.join(MANIFEST)).unwrap()).unwrap();
            assert_eq!(manifest.scenes.len(), count);
            assert!(manifest.scenes.iter().any(|s| s.name == "Unsaved other"));
            assert_eq!(fs::read(&other_path).unwrap(), saved);
            assert!(!build.join("epok.ps-exe").exists());
        }
        drop(project);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn symbol_attribution_respects_resource_and_class_boundaries() {
        assert!(symbol_matches("epok::audio_data_1", "epok::audio_data_1"));
        assert!(!symbol_matches("epok::audio_data_10", "epok::audio_data_1"));
        assert!(symbol_matches(
            "Player::update(epok::Transform&)",
            "Player::"
        ));
        assert!(!symbol_matches("OtherPlayer::update()", "Player::"));
        assert!(!symbol_matches("other::Player::update()", "Player::"));
    }
    #[test]
    fn aliases_padding_and_bss_are_counted_once() {
        let section = Section {
            name: ".bss".into(),
            address: 0x80010000,
            size: 4096,
            flags: 3,
            kind: 8,
        };
        let symbols = vec![
            Symbol {
                name: "pool".into(),
                address: 0x80010010,
                size: 2048,
                section: 1,
            },
            Symbol {
                name: "alias".into(),
                address: 0x80010010,
                size: 2048,
                section: 1,
            },
            Symbol {
                name: "partial_alias".into(),
                address: 0x80010800,
                size: 64,
                section: 1,
            },
        ];
        let nodes = section_nodes(1, &section, &symbols, &[], &Default::default());
        assert_eq!(nodes.iter().map(|(_, n)| n.bytes).sum::<u64>(), 4096);
        assert_eq!(
            nodes
                .iter()
                .filter(|(c, _)| c == "Pools / runtime state")
                .map(|(_, n)| n.bytes)
                .sum::<u64>(),
            2096
        );
        let space = Space::new("RAM", Some(8192), merge(nodes), "Unassigned");
        assert_eq!(space.used, 4096);
        assert_eq!(space.root.bytes, 8192);
    }
    #[test]
    fn malformed_elf_is_rejected_without_panics() {
        for length in 0..80 {
            assert!(Elf::parse(&vec![0; length]).is_err());
        }
        assert!(bytes(&[0; 8], u64::MAX, 2).is_err());
        assert!(string(b"unterminated", 0).is_err());
    }
    #[test]
    fn overflow_is_preserved_and_shared_resources_merge() {
        let node = Node::leaf("Shared texture", 128, "resident");
        let nodes = merge(vec![
            ("Textures".into(), node.clone()),
            ("Textures".into(), node),
        ]);
        assert_eq!(nodes[0].children.len(), 1);
        let space = Space::new("RAM", Some(200), nodes, "Unassigned");
        assert_eq!(space.used, 256);
        assert_eq!(space.root.bytes, 256);
    }
}
