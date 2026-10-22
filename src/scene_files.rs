//! External scene files. A scene marked to load from disc keeps its cooked data
//! out of the executable: the linker places the data of every external bank in
//! one shared RAM region (an overlay), and each bank's image is saved as
//! SCENEnn.BIN. The runtime reads that file from the CD or through PCDrv when
//! the scene starts (runtime/scene_files.hpp). Scripts and engine code always
//! stay in the executable; only data in the bank's `epok::scene_N` namespace
//! moves, plus textures that no resident bank uses.
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const MANIFEST: &str = "scene-files.json";
const MAKE_FRAGMENT: &str = "scene-overlays.mk";
const LINKER_SCRIPT: &str = "scene-overlays.ld";
/// `epok::scene_file_records`, written after linking with each file's size and hash.
const RECORDS: &str = "_ZN4epok18scene_file_recordsE";
/// Unpatched records keep the table in `.data` and fail at run time if loaded.
const PLACEHOLDER: [u32; 2] = [0xffff_ffff, 0x4550_4f4b];
/// First word of every scene image, followed by its bank index.
const MAGIC: u32 = 0x454e_4353;
const SECTOR: usize = 2048;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<File>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    pub bank: usize,
    pub scene: String,
    pub name: String,
}

pub fn file_name(bank: usize) -> String {
    format!("SCENE{bank:02}.BIN")
}
pub fn is_file_name(name: &str) -> bool {
    name.len() == 11
        && name.starts_with("SCENE")
        && name.ends_with(".BIN")
        && name[5..7].bytes().all(|b| b.is_ascii_digit())
}
fn section(bank: usize) -> String {
    format!(".epok_scene_{bank}")
}
/// Itanium-mangled prefix of every entity in `epok::scene_<bank>`.
fn mangled(bank: usize) -> String {
    let name = format!("scene_{bank}");
    format!("N4epok{}{name}", name.len())
}

/// Project-relative map path, as stored in the scene table.
pub fn origin_path(root: &Path, origin: &crate::scene_dependencies::Origin) -> Option<String> {
    let path = match origin {
        crate::scene_dependencies::Origin::Saved(path, _)
        | crate::scene_dependencies::Origin::Editor(path, _) => path,
        crate::scene_dependencies::Origin::Anonymous(_) => return None,
    };
    let root = fs::canonicalize(root).ok()?;
    let path = fs::canonicalize(path).ok()?;
    Some(
        path.strip_prefix(root)
            .ok()?
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

/// Which banks load from disc. Only PlayStation builds that read data from the
/// CD or PCDrv have somewhere to load them from; every other build keeps all
/// scenes in the executable. Projects without a scene table keep theirs too.
pub fn plan(
    storage: Option<&crate::play::SceneStorage>,
    profile: Option<&crate::play::Profile>,
    paths: &[Option<String>],
) -> Vec<bool> {
    let loads_files = profile.is_some_and(|p| {
        p.runtime == crate::play::Runtime::PlayStation
            && matches!(
                p.data,
                crate::play::DataSource::Disc | crate::play::DataSource::Host
            )
    });
    paths
        .iter()
        .map(|path| match (storage, path) {
            (Some(storage), Some(path)) if loads_files => !storage.contains(path),
            _ => false,
        })
        .collect()
}

/// Records, names and the count the runtime needs, inside `namespace epok`.
pub fn header(external: &[bool]) -> String {
    let banks: Vec<_> = (0..external.len()).filter(|&b| external[b]).collect();
    if banks.is_empty() {
        return "inline constexpr size_t scene_file_count=0;\ninline constexpr const char* scene_file_names[1]={nullptr};\ninline constexpr const char* scene_file_paths[1]={nullptr};\ninline SceneFileRecord scene_file_records[1]={};\n".into();
    }
    let list = |suffix: &str| {
        banks
            .iter()
            .map(|&b| format!("\"{}{suffix}\"", file_name(b)))
            .collect::<Vec<_>>()
            .join(",")
    };
    let records = banks
        .iter()
        .map(|_| format!("{{{}u,{}u}}", PLACEHOLDER[0], PLACEHOLDER[1]))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "inline constexpr size_t scene_file_count={};\ninline constexpr const char* scene_file_names[]={{{}}};\ninline constexpr const char* scene_file_paths[]={{{}}};\n// Written by the editor after linking: size and FNV-1a hash of each file.\n__attribute__((used)) inline SceneFileRecord scene_file_records[]={{{records}}};\n",
        banks.len(),
        list(""),
        list(";1")
    )
}
/// File index of each bank in `scene_file_records`, or -1 for a resident bank.
pub fn indices(external: &[bool]) -> Vec<i32> {
    let mut next = 0;
    external
        .iter()
        .map(|&e| {
            if e {
                next += 1;
                next - 1
            } else {
                -1
            }
        })
        .collect()
}

/// Every external bank links at the same address after the BSS, which crt0
/// clears and the heap follows. Load addresses lie outside RAM so that no
/// loaded section follows them: objcopy drops these sections from the PS-X EXE.
/// Inputs are the bank's namespace-scope data (mangled names end in `E`) and
/// the local statics of its initialize_components. String literals stay in
/// the executable: GCC shares identical literals between banks of one unit.
fn linker_script(banks: &[usize]) -> String {
    let mut out = String::from(
        "/* Generated by Epok: external scene data. Every bank below links at the same\n   address; the runtime reads one SCENEnn.BIN there before the bank starts. */\n__heap_base = __bss_end;\nSECTIONS {\n  . = ALIGN(2048);\n  __epok_scene_region_start = .;\n",
    );
    let mut load = "0x81000000".to_owned();
    for &bank in banks {
        let prefix = mangled(bank);
        let inputs = [".rodata", ".data", ".sdata", ".bss", ".sbss"]
            .iter()
            .map(|kind| {
                format!("{kind}._Z{prefix}*E {kind}._ZZ{prefix}21initialize_componentsEvE[0-9]*")
            })
            .collect::<Vec<_>>()
            .join(" ");
        let name = section(bank);
        out += &format!(
            "  {name} __epok_scene_region_start : AT({load}) {{\n    LONG(0x{MAGIC:08x}); LONG({bank});\n    *({inputs})\n    . = ALIGN(2048);\n  }}\n"
        );
        load = format!("LOADADDR({name}) + SIZEOF({name})");
    }
    let largest = banks.iter().fold("0".to_owned(), |size, &bank| {
        format!("MAX({size}, SIZEOF({}))", section(bank))
    });
    out += &format!(
        "  . = __epok_scene_region_start + {largest};\n  __epok_scene_region_end = .;\n  /* Only R_MIPS_NONE entries; placed after the region it would enlarge the EXE. */\n  /DISCARD/ : {{ *(.rel.dyn) }}\n}}\n"
    );
    if banks.len() > 1 {
        out += &format!(
            "NOCROSSREFS({})\n",
            banks
                .iter()
                .map(|&b| section(b))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    out += "INSERT AFTER .sbss;\n";
    out
}

/// Write the overlay link inputs and the list of scene files for this build.
/// Builds without external scenes get an empty make fragment, never a stale one.
pub fn stage(build: &Path, banks: &[crate::scene::Scene], external: &[bool]) -> Result<(), String> {
    let manifest = Manifest {
        files: external
            .iter()
            .enumerate()
            .filter(|(_, e)| **e)
            .map(|(bank, _)| File {
                bank,
                scene: banks[bank].name.clone(),
                name: file_name(bank),
            })
            .collect(),
    };
    let list: Vec<_> = manifest.files.iter().map(|f| f.bank).collect();
    if list.is_empty() {
        crate::project::write_changed(
            &build.join(MAKE_FRAGMENT),
            b"# Generated by Epok: every scene keeps its data in the executable.\n",
        )?;
        remove(&build.join(LINKER_SCRIPT))?;
    } else {
        crate::project::write_changed(&build.join(LINKER_SCRIPT), linker_script(&list).as_bytes())?;
        crate::project::write_changed(
            &build.join(MAKE_FRAGMENT),
            format!(
                "# Generated by Epok: link external scene data as overlays.\nOVERLAYSCRIPT := {LINKER_SCRIPT}\nOVERLAYSECTION := {}\nEXTRA_DEPS += {LINKER_SCRIPT}\n",
                list.iter().map(|&b| section(b)).collect::<Vec<_>>().join(" ")
            )
            .as_bytes(),
        )?;
    }
    // Files from an earlier configuration must not reach a disc or package.
    for entry in fs::read_dir(build).map_err(|e| e.to_string())? {
        let name = entry.map_err(|e| e.to_string())?.file_name();
        let name = name.to_string_lossy();
        if is_file_name(&name) && !manifest.files.iter().any(|f| f.name == name) {
            remove(&build.join(name.as_ref()))?;
        }
    }
    crate::project::write_changed(
        &build.join(MANIFEST),
        &serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
}
fn remove(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    }
}

/// The scene files this build's staging selected; empty when none.
pub fn staged(build: &Path) -> Vec<File> {
    fs::read(build.join(MANIFEST))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Manifest>(&bytes).ok())
        .map(|m| m.files)
        .unwrap_or_default()
}

fn hash(bytes: &[u8]) -> u32 {
    crate::streaming::page_hash(bytes)
}

struct Linked {
    elf: Vec<u8>,
    records_offset: usize,
    records_address: u64,
    region: (u64, u64),
    images: Vec<Vec<u8>>,
}
fn linked(build: &Path, files: &[File]) -> Result<Linked, String> {
    let elf = fs::read(build.join("epok.elf")).map_err(|e| format!("Scene files: {e}"))?;
    let parsed = crate::memory::Elf::parse(&elf)?;
    // Data that needs a static constructor would be initialized at boot, before
    // its scene file is even read, and then overwritten by the file.
    for file in files {
        let prefix = mangled(file.bank);
        if parsed.symbols.iter().any(|s| {
            s.name.starts_with(&format!("_ZGV{prefix}"))
                || s.name.starts_with(&format!("_ZGVZ{prefix}"))
        }) {
            return Err(format!(
                "Scene {} has data that is initialized at startup and cannot load from disc. Keep it in the executable.",
                file.scene
            ));
        }
    }
    let symbol = |name: &str| parsed.symbols.iter().find(|s| s.name == name);
    let region = match (
        symbol("__epok_scene_region_start"),
        symbol("__epok_scene_region_end"),
    ) {
        (Some(start), Some(end)) if end.address > start.address => (start.address, end.address),
        _ => return Err("The linked executable has no scene file region.".into()),
    };
    let mut images = Vec::new();
    for file in files {
        let name = section(file.bank);
        let section = parsed
            .sections
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| {
                format!(
                    "The linked executable has no data for scene {}.",
                    file.scene
                )
            })?;
        let start = usize::try_from(section.offset).map_err(|e| e.to_string())?;
        let size = usize::try_from(section.size).map_err(|e| e.to_string())?;
        let bytes = elf
            .get(start..start + size)
            .filter(|_| section.kind != 8)
            .ok_or("Truncated scene section in the linked executable")?;
        if section.address != region.0
            || section.address + section.size > region.1
            || size < 8
            || size % SECTOR != 0
            || u32::from_le_bytes(bytes[..4].try_into().unwrap()) != MAGIC
            || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize != file.bank
        {
            return Err(format!(
                "Scene {} was not linked into the scene file region.",
                file.scene
            ));
        }
        images.push(bytes.to_vec());
    }
    let records = symbol(RECORDS).ok_or("The linked executable has no scene file table.")?;
    let section = parsed
        .sections
        .get(records.section)
        .filter(|s| s.kind != 8)
        .ok_or("The scene file table is not stored in the executable.")?;
    if records.size != 8 * files.len() as u64 {
        return Err("The scene file table does not match the scene files.".into());
    }
    let records_offset = usize::try_from(section.offset + (records.address - section.address))
        .map_err(|e| e.to_string())?;
    Ok(Linked {
        elf,
        records_offset,
        records_address: records.address,
        region,
        images,
    })
}
fn executable_offset(exe: &[u8], address: u64, bytes: usize) -> Result<usize, String> {
    if !exe.starts_with(b"PS-X EXE") || exe.len() < 0x800 {
        return Err("Expected a PS-X EXE".into());
    }
    let load = u64::from(u32::from_le_bytes(exe[0x18..0x1c].try_into().unwrap()));
    let size = u64::from(u32::from_le_bytes(exe[0x1c..0x20].try_into().unwrap()));
    if address < load || address + bytes as u64 > load + size {
        return Err("The scene file table is outside the executable image.".into());
    }
    let offset = usize::try_from(0x800 + address - load).map_err(|e| e.to_string())?;
    if offset + bytes > exe.len() {
        return Err("Truncated PS-X EXE".into());
    }
    Ok(offset)
}

/// After linking: save each external bank's image as its scene file and write
/// the files' sizes and hashes into the ELF and the PS-X EXE. Unchanged outputs
/// are left untouched, so an unchanged build reproduces the same bytes.
pub fn finish_link(build: &Path) -> Result<(), String> {
    let files = staged(build);
    if files.is_empty() {
        return Ok(());
    }
    let mut linked = linked(build, &files)?;
    let mut table = Vec::new();
    for (file, image) in files.iter().zip(&linked.images) {
        crate::project::write_changed(&build.join(&file.name), image)?;
        table.extend((image.len() as u32).to_le_bytes());
        table.extend(hash(image).to_le_bytes());
    }
    let exe_path = build.join("epok.ps-exe");
    let mut exe = fs::read(&exe_path).map_err(|e| format!("Missing PSX executable: {e}"))?;
    let load = u64::from(u32::from_le_bytes(
        exe.get(0x18..0x1c)
            .ok_or("Truncated PS-X EXE")?
            .try_into()
            .unwrap(),
    ));
    let size = u64::from(u32::from_le_bytes(exe[0x1c..0x20].try_into().unwrap()));
    // Neither the region nor anything after it may reach the EXE file.
    if linked.region.0 < load + size || exe.len() as u64 != 0x800 + size {
        return Err("Scene file data was linked into the executable image.".into());
    }
    let offset = executable_offset(&exe, linked.records_address, table.len())?;
    let range = linked.records_offset..linked.records_offset + table.len();
    // The EXE is a copy of the ELF's loaded image: both must agree before patching.
    if exe[offset..offset + table.len()] != linked.elf[range.clone()] {
        return Err("The PS-X EXE does not match the linked ELF.".into());
    }
    if linked.elf[range.clone()] != table[..] {
        linked.elf[range].copy_from_slice(&table);
        fs::write(build.join("epok.elf"), &linked.elf).map_err(|e| e.to_string())?;
        exe[offset..offset + table.len()].copy_from_slice(&table);
        fs::write(&exe_path, &exe).map_err(|e| e.to_string())?;
    }
    verify(build, &exe)
}

/// Every scene file of this build must match the record in this executable.
pub fn verify(build: &Path, exe: &[u8]) -> Result<(), String> {
    let files = staged(build);
    if files.is_empty() {
        return Ok(());
    }
    let elf = fs::read(build.join("epok.elf")).map_err(|e| format!("Scene files: {e}"))?;
    let parsed = crate::memory::Elf::parse(&elf)?;
    let records = parsed
        .symbols
        .iter()
        .find(|s| s.name == RECORDS)
        .ok_or("The linked executable has no scene file table.")?;
    let offset = executable_offset(exe, records.address, 8 * files.len())?;
    for (i, file) in files.iter().enumerate() {
        let bytes = fs::read(build.join(&file.name))
            .map_err(|e| format!("Scene file {} for {}: {e}", file.name, file.scene))?;
        let at = offset + 8 * i;
        let size = u32::from_le_bytes(exe[at..at + 4].try_into().unwrap());
        let expected = u32::from_le_bytes(exe[at + 4..at + 8].try_into().unwrap());
        if size as usize != bytes.len() || expected != hash(&bytes) {
            return Err(format!(
                "{} does not match the executable it was built with. Build again.",
                file.name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::{DataSource, Profile, Runtime, SceneStorage};

    #[test]
    fn only_disc_and_pcdrv_playstation_builds_load_scenes_from_files() {
        let storage = SceneStorage {
            in_executable: vec!["assets/scenes/Menu.epokmap".into()],
        };
        let paths = [
            Some("assets/scenes/Menu.epokmap".to_owned()),
            Some("assets/scenes/Level.epokmap".to_owned()),
            None,
        ];
        let profile = |data| Profile {
            data,
            ..Default::default()
        };
        for data in [DataSource::Disc, DataSource::Host] {
            assert_eq!(
                plan(Some(&storage), Some(&profile(data)), &paths),
                [false, true, false],
                "{data:?}"
            );
        }
        assert_eq!(
            plan(
                Some(&storage),
                Some(&profile(DataSource::Executable)),
                &paths
            ),
            [false; 3]
        );
        let native = Profile {
            runtime: Runtime::NativePc,
            ..profile(DataSource::Disc)
        };
        assert_eq!(plan(Some(&storage), Some(&native), &paths), [false; 3]);
        // A project that predates the scene table keeps every scene resident.
        assert_eq!(
            plan(None, Some(&profile(DataSource::Disc)), &paths),
            [false; 3]
        );
        assert_eq!(plan(Some(&storage), None, &paths), [false; 3]);
    }

    #[test]
    fn overlay_script_routes_only_each_banks_namespace() {
        let script = linker_script(&[1, 12]);
        assert!(script.contains(
            ".rodata._ZN4epok7scene_1*E .rodata._ZZN4epok7scene_121initialize_componentsEvE[0-9]*"
        ));
        assert!(script.contains(".data._ZN4epok8scene_12*E"));
        assert!(script.contains(".epok_scene_1 __epok_scene_region_start : AT(0x81000000) {"));
        assert!(script.contains(
            ".epok_scene_12 __epok_scene_region_start : AT(LOADADDR(.epok_scene_1) + SIZEOF(.epok_scene_1)) {"
        ));
        assert!(script.contains("LONG(0x454e4353); LONG(12);"));
        assert!(script.contains("MAX(MAX(0, SIZEOF(.epok_scene_1)), SIZEOF(.epok_scene_12))"));
        assert!(script.contains("NOCROSSREFS(.epok_scene_1 .epok_scene_12)"));
        assert!(script.ends_with("INSERT AFTER .sbss;\n"));
        // scene_1's pattern cannot capture scene_12: the name length differs.
        assert!(!"_ZN4epok8scene_123fooE".starts_with("_ZN4epok7scene_1"));
        // Code and string literals stay in the executable.
        assert!(!script.contains(".text"));
        assert!(!script.contains(".str"));
        assert!(!linker_script(&[3]).contains("NOCROSSREFS"));
    }

    #[test]
    fn header_numbers_files_in_bank_order_and_keeps_records_patchable() {
        let external = [false, true, false, true];
        assert_eq!(indices(&external), [-1, 0, -1, 1]);
        let text = header(&external);
        assert!(text.contains("scene_file_count=2;"));
        assert!(text.contains("scene_file_names[]={\"SCENE01.BIN\",\"SCENE03.BIN\"}"));
        assert!(text.contains("scene_file_paths[]={\"SCENE01.BIN;1\",\"SCENE03.BIN;1\"}"));
        let record = format!("{{{}u,{}u}}", PLACEHOLDER[0], PLACEHOLDER[1]);
        assert!(text.contains(&format!("{{{record},{record}}}")));
        assert!(header(&[false]).contains("scene_file_count=0;"));
        assert!(is_file_name("SCENE03.BIN"));
        for name in [
            "SCENE3.BIN",
            "scene03.bin",
            "SCENE03.BIN.bak",
            "GEOMETRY.BIN",
        ] {
            assert!(!is_file_name(name), "{name}");
        }
    }
}
