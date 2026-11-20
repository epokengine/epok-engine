//! Win32 resources linked into the locally compiled native runners.
//!
//! `runner.exe` is compiled on the user's machine, so it is unsigned and every
//! build has a new hash. A bare MSVC executable with no version block, manifest
//! or icon is exactly what generic antivirus heuristics flag as a potentially
//! unwanted application (Bitdefender reported `Gen:Variant.Application.Cerbu`).
//! These resources give the runner the same identity the editor carries.
//!
//! The `.res` file is produced here rather than by `rc.exe`: the Windows SDK
//! resource compiler is not guaranteed to be on the build-tools PATH, while
//! `link.exe` accepts a compiled `.res` directly.

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const RT_VERSION: u16 = 16;
const RT_MANIFEST: u16 = 24;
/// en-US, matching the `040904B0` string table below.
const LANGUAGE: u16 = 0x0409;
const CODEPAGE_UNICODE: u16 = 1200;

const ICON: &[u8] = include_bytes!("../resources/branding/epok.ico");

/// Matches what `link /MANIFEST:EMBED` would generate, plus the Windows 10/11
/// compatibility declaration. No `assemblyIdentity`: a malformed one stops the
/// process from starting with a side-by-side error, and none is needed here.
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>
"#;

/// A complete `.res` file: version block, application manifest and the Epok icon.
pub(crate) fn runner_res(description: &str) -> Vec<u8> {
    let mut res = Vec::new();
    // Every .res file begins with an empty entry that identifies the format.
    entry(&mut res, 0, 0, 0, 0, &[]);
    entry(
        &mut res,
        RT_VERSION,
        1,
        LANGUAGE,
        0x0030,
        &version_info(description),
    );
    entry(
        &mut res,
        RT_MANIFEST,
        1,
        LANGUAGE,
        0x0030,
        MANIFEST.as_bytes(),
    );
    icons(&mut res);
    res
}

fn version() -> [u16; 4] {
    let part = |value: &str| value.parse().unwrap_or(0);
    [
        part(env!("CARGO_PKG_VERSION_MAJOR")),
        part(env!("CARGO_PKG_VERSION_MINOR")),
        part(env!("CARGO_PKG_VERSION_PATCH")),
        0,
    ]
}

fn pad(bytes: &mut Vec<u8>) {
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
}

fn wide(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// One `RESOURCEHEADER` with ordinal type and name, followed by its data.
fn entry(res: &mut Vec<u8>, kind: u16, name: u16, language: u16, flags: u16, data: &[u8]) {
    res.extend((data.len() as u32).to_le_bytes());
    res.extend(32u32.to_le_bytes());
    res.extend(
        [0xFFFF, kind, 0xFFFF, name]
            .iter()
            .flat_map(|w| w.to_le_bytes()),
    );
    res.extend(0u32.to_le_bytes()); // DataVersion
    res.extend(flags.to_le_bytes());
    res.extend(language.to_le_bytes());
    res.extend(0u32.to_le_bytes()); // Version
    res.extend(0u32.to_le_bytes()); // Characteristics
    res.extend(data);
    pad(res);
}

/// A version-information node: `wLength`, `wValueLength`, `wType`, key, value,
/// then DWORD-aligned children. `wLength` covers everything but trailing padding.
fn node(key: &str, value_length: u16, text: bool, value: &[u8], children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 0];
    bytes.extend(value_length.to_le_bytes());
    bytes.extend(u16::from(text).to_le_bytes());
    bytes.extend(wide(key));
    pad(&mut bytes);
    bytes.extend(value);
    for child in children {
        pad(&mut bytes);
        bytes.extend(child);
    }
    let length = bytes.len() as u16;
    bytes[..2].copy_from_slice(&length.to_le_bytes());
    bytes
}

fn string(key: &str, value: &str) -> Vec<u8> {
    let value = wide(value);
    // String values count UTF-16 units, including the terminator.
    node(key, (value.len() / 2) as u16, true, &value, &[])
}

fn version_info(description: &str) -> Vec<u8> {
    let [major, minor, patch, build] = version();
    let ms = (u32::from(major) << 16) | u32::from(minor);
    let ls = (u32::from(patch) << 16) | u32::from(build);
    let mut fixed = Vec::with_capacity(52);
    for word in [
        0xFEEF_04BD, // signature
        0x0001_0000, // structure version
        ms,
        ls, // file version
        ms,
        ls,          // product version
        0x3F,        // VS_FFI_FILEFLAGSMASK
        0,           // flags: release, not pre-release
        0x0004_0004, // VOS_NT_WINDOWS32
        1,           // VFT_APP
        0,
        0,
        0, // subtype, date
    ] {
        fixed.extend(u32::to_le_bytes(word));
    }
    let text = format!("{major}.{minor}.{patch}.{build}");
    let strings = node(
        &format!("{LANGUAGE:04X}{CODEPAGE_UNICODE:04X}"),
        0,
        true,
        &[],
        &[
            string("CompanyName", "Epok Engine"),
            string("FileDescription", description),
            string("FileVersion", &text),
            string("InternalName", "runner"),
            string("LegalCopyright", "Generated by Epok Engine (MIT)"),
            string("OriginalFilename", "runner.exe"),
            string("ProductName", "Epok Engine"),
            string("ProductVersion", &text),
        ],
    );
    let translation = [LANGUAGE.to_le_bytes(), CODEPAGE_UNICODE.to_le_bytes()].concat();
    node(
        "VS_VERSION_INFO",
        fixed.len() as u16,
        false,
        &fixed,
        &[
            node("StringFileInfo", 0, true, &[], &[strings]),
            node(
                "VarFileInfo",
                0,
                true,
                &[],
                &[node("Translation", 4, false, &translation, &[])],
            ),
        ],
    )
}

/// The `.ico` images become `RT_ICON` entries 1..n, indexed by one `RT_GROUP_ICON`.
fn icons(res: &mut Vec<u8>) {
    let word = |at: usize| u16::from_le_bytes([ICON[at], ICON[at + 1]]);
    let dword = |at: usize| u32::from_le_bytes(ICON[at..at + 4].try_into().unwrap());
    let count = word(4);
    let mut group = ICON[..6].to_vec();
    for index in 0..count {
        let at = 6 + 16 * usize::from(index);
        let (size, offset) = (dword(at + 8) as usize, dword(at + 12) as usize);
        entry(
            res,
            RT_ICON,
            index + 1,
            LANGUAGE,
            0x1010,
            &ICON[offset..offset + size],
        );
        // GRPICONDIRENTRY is ICONDIRENTRY with the file offset replaced by an id.
        group.extend(&ICON[at..at + 12]);
        group.extend((index + 1).to_le_bytes());
    }
    entry(res, RT_GROUP_ICON, 1, LANGUAGE, 0x1030, &group);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Entry {
        kind: u16,
        name: u16,
        data: Vec<u8>,
    }

    fn entries(res: &[u8]) -> Vec<Entry> {
        let mut at = 0;
        let mut out = vec![];
        while at < res.len() {
            let size = u32::from_le_bytes(res[at..at + 4].try_into().unwrap()) as usize;
            let header = u32::from_le_bytes(res[at + 4..at + 8].try_into().unwrap()) as usize;
            assert_eq!(header, 32, "ordinal headers are always 32 bytes");
            assert_eq!(&res[at + 8..at + 10], &[0xFF, 0xFF]);
            assert_eq!(&res[at + 12..at + 14], &[0xFF, 0xFF]);
            let kind = u16::from_le_bytes([res[at + 10], res[at + 11]]);
            let name = u16::from_le_bytes([res[at + 14], res[at + 15]]);
            out.push(Entry {
                kind,
                name,
                data: res[at + header..at + header + size].to_vec(),
            });
            at += header + size;
            at += (4 - at % 4) % 4;
        }
        assert_eq!(at, res.len());
        out
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn runner_resources_form_a_well_formed_res_file() {
        let res = runner_res("Epok native PC play runner");
        let entries = entries(&res);
        assert_eq!(
            (entries[0].kind, entries[0].name, entries[0].data.len()),
            (0, 0, 0)
        );

        let version = &entries.iter().find(|e| e.kind == RT_VERSION).unwrap().data;
        assert_eq!(
            usize::from(u16::from_le_bytes([version[0], version[1]])),
            version.len()
        );
        assert_eq!(u16::from_le_bytes([version[2], version[3]]), 52);
        // "VS_VERSION_INFO\0" is 32 bytes after the 6-byte prefix: value at 40.
        assert_eq!(&version[40..44], &0xFEEF_04BDu32.to_le_bytes());
        let [major, minor, ..] = super::version();
        assert_eq!(
            u32::from_le_bytes(version[48..52].try_into().unwrap()),
            (u32::from(major) << 16) | u32::from(minor)
        );
        for text in [
            "FileDescription",
            "Epok native PC play runner",
            "040904B0",
            "Translation",
        ] {
            assert!(contains(version, &wide(text)[..text.len() * 2]), "{text}");
        }

        let manifest = &entries.iter().find(|e| e.kind == RT_MANIFEST).unwrap();
        assert_eq!(manifest.name, 1, "CREATEPROCESS_MANIFEST_RESOURCE_ID");
        assert!(contains(&manifest.data, b"level=\"asInvoker\""));

        let group = &entries
            .iter()
            .find(|e| e.kind == RT_GROUP_ICON)
            .unwrap()
            .data;
        let count = u16::from_le_bytes([group[4], group[5]]);
        assert!(count > 0);
        assert_eq!(group.len(), 6 + 14 * usize::from(count));
        let icons = entries.iter().filter(|e| e.kind == RT_ICON).count();
        assert_eq!(icons, usize::from(count));
    }
}
